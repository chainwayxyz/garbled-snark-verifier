//! Garbler-side orchestration for the cut-and-choose Setup phase described in
//! `docs/gsv_spec.md`. The API mirrors the protocol steps: `commit_phase_one`
//! produces the `Commit₁` payload, `commit_phase_two` covers `Commit₂`, and
//! `open_commit` implements the challenge/opening flow.
use std::{
    mem,
    thread::{self, JoinHandle},
};

use rand::Rng;
use rayon::{iter::IntoParallelRefIterator, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::info;

#[cfg(feature = "sp1-soldering")]
use crate::sp1_soldering::{self, SolderingProof};
use crate::{
    AESAccumulatingHash, AesNiHasher, GarbleMode, GarbledWire, S, WireId,
    circuit::{
        CiphertextHandler, CircuitBuilder, CircuitInput, EncodeInput, StreamingMode,
        StreamingResult,
    },
    cut_and_choose::{
        CiphertextCommit, Config, DefaultLabelCommitHasher, LabelCommit, LabelCommitHasher, Seed,
        commit_label_with,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GarbledInstance {
    /// Constant to represent false wire constant
    ///
    /// Necessary to restart the scheme and consistency
    pub false_wire_constant: GarbledWire,

    /// Constant to represent true wire constant
    ///
    /// Necessary to restart the scheme and consistency
    pub true_wire_constant: GarbledWire,

    /// Output `WireId` in return order
    pub output_wire_values: GarbledWire,

    /// Values of the input Wires, which were fed to the circuit input
    pub input_wire_values: Vec<GarbledWire>,

    pub ciphertext_handler_result: CiphertextCommit,
}

impl<I: CircuitInput>
    From<StreamingResult<GarbleMode<AesNiHasher, AESAccumulatingHash>, I, GarbledWire>>
    for GarbledInstance
{
    fn from(
        res: StreamingResult<GarbleMode<AesNiHasher, AESAccumulatingHash>, I, GarbledWire>,
    ) -> Self {
        GarbledInstance {
            false_wire_constant: res.false_wire_constant,
            true_wire_constant: res.true_wire_constant,
            output_wire_values: res.output_value,
            input_wire_values: res.input_wire_values,
            ciphertext_handler_result: res.ciphertext_handler_result,
        }
    }
}

/// `Commit₁(i)` payload containing ciphertext hash, per-wire input commits,
/// output commits, and constant wire values (spec Step 1.2).
#[derive(Debug, Serialize, Deserialize, Eq)]
#[serde(bound = "H: LabelCommitHasher")]
pub struct CommitPhaseOne<H: LabelCommitHasher = DefaultLabelCommitHasher> {
    ciphertext_hash: CiphertextCommit,
    input_commitments: Vec<LabelCommit<H::Output>>,
    /// Commitment to the active output label when the circuit output is `true`.
    output_label1_commit: H::Output,
    /// Commitment to the active output label when the circuit output is `false`.
    output_label0_commit: H::Output,
    true_constant: u128,
    false_constant: u128,
}

impl<H: LabelCommitHasher> Clone for CommitPhaseOne<H> {
    fn clone(&self) -> Self {
        Self {
            ciphertext_hash: self.ciphertext_hash,
            input_commitments: self.input_commitments.clone(),
            output_label0_commit: self.output_label0_commit,
            output_label1_commit: self.output_label1_commit,
            true_constant: self.true_constant,
            false_constant: self.false_constant,
        }
    }
}

impl<H: LabelCommitHasher> PartialEq for CommitPhaseOne<H> {
    fn eq(&self, other: &Self) -> bool {
        self.ciphertext_hash == other.ciphertext_hash
            && self.input_commitments == other.input_commitments
            && self.output_label1_commit == other.output_label1_commit
            && self.output_label0_commit == other.output_label0_commit
            && self.true_constant == other.true_constant
            && self.false_constant == other.false_constant
    }
}

impl<H: LabelCommitHasher> CommitPhaseOne<H> {
    /// Create a new `CommitPhaseOne` directly from its components.
    pub fn new(
        ciphertext_hash: CiphertextCommit,
        input_commitments: Vec<LabelCommit<H::Output>>,
        output_label1_commit: H::Output,
        output_label0_commit: H::Output,
        true_constant: u128,
        false_constant: u128,
    ) -> Self {
        Self {
            ciphertext_hash,
            input_commitments,
            output_label1_commit,
            output_label0_commit,
            true_constant,
            false_constant,
        }
    }

    /// Recompute the `Commit₁` payload (without nonce injection) for a garbled instance.
    pub fn from_instance(instance: &GarbledInstance) -> Self {
        Self {
            ciphertext_hash: instance.ciphertext_handler_result,
            input_commitments: commit_input_wires::<H>(&instance.input_wire_values, None),
            output_label1_commit: commit_output_label1::<H>(&instance.output_wire_values),
            output_label0_commit: commit_output_label0::<H>(&instance.output_wire_values),
            true_constant: instance.true_wire_constant.select(true).to_u128(),
            false_constant: instance.false_wire_constant.select(false).to_u128(),
        }
    }

    pub fn ciphertext_hash(&self) -> CiphertextCommit {
        self.ciphertext_hash
    }

    pub fn input_commitments(&self) -> &[LabelCommit<H::Output>] {
        &self.input_commitments
    }

    pub fn output_commit_true(&self) -> H::Output {
        self.output_label1_commit
    }

    pub fn output_commit_false(&self) -> H::Output {
        self.output_label0_commit
    }

    pub fn true_constant(&self) -> u128 {
        self.true_constant
    }

    pub fn false_constant(&self) -> u128 {
        self.false_constant
    }
}

/// `Commit₂(i)` payload containing nonce-blended per-wire input commitments
/// (spec Step 1.4).
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound = "H: LabelCommitHasher")]
pub struct CommitPhaseTwo<H: LabelCommitHasher = DefaultLabelCommitHasher> {
    input_commitments: Vec<LabelCommit<H::Output>>,
}

impl<H: LabelCommitHasher> Clone for CommitPhaseTwo<H> {
    fn clone(&self) -> Self {
        Self {
            input_commitments: self.input_commitments.clone(),
        }
    }
}

impl<H: LabelCommitHasher> CommitPhaseTwo<H> {
    /// Create a new `CommitPhaseTwo` directly from input commitments.
    pub fn new(input_commitments: Vec<LabelCommit<H::Output>>) -> Self {
        Self { input_commitments }
    }

    /// Recompute the `Commit₂` payload (with nonce injection) for a garbled instance.
    pub fn from_instance(instance: &GarbledInstance, nonce: S) -> Self {
        Self {
            input_commitments: commit_input_wires::<H>(&instance.input_wire_values, Some(nonce)),
        }
    }

    pub fn input_commitments(&self) -> &[LabelCommit<H::Output>] {
        &self.input_commitments
    }

    pub fn into_inner(self) -> Vec<LabelCommit<H::Output>> {
        self.input_commitments
    }
}

fn commit_output_label1<H: LabelCommitHasher>(wire: &GarbledWire) -> H::Output {
    commit_label_with::<H>(wire.label1)
}

fn commit_output_label0<H: LabelCommitHasher>(wire: &GarbledWire) -> H::Output {
    commit_label_with::<H>(wire.label0)
}

fn commit_input_wires<H: LabelCommitHasher>(
    inputs: &[GarbledWire],
    nonce: Option<S>,
) -> Vec<LabelCommit<H::Output>> {
    inputs
        .iter()
        .map(|GarbledWire { label0, label1 }| {
            LabelCommit::<H::Output>::new::<H>(*label0, *label1, &nonce)
        })
        .collect()
}

pub enum OpenForInstance {
    Open(usize, Seed),
    Closed {
        index: usize,
        garbling_thread: JoinHandle<()>,
    },
}

/// Result of opening commitments without ciphertext handlers
#[derive(Debug)]
pub struct OpenCommit {
    pub open: Vec<(usize, Seed)>,
    pub closed: Vec<(usize, Seed)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GarblerStage {
    Generating { seeds: Box<[Seed]> },
    PreparedForEval { indexes_to_eval: Box<[usize]> },
}

impl GarblerStage {
    fn next_stage(&mut self, indexes_to_eval: Box<[usize]>) -> Box<[Seed]> {
        assert!(matches!(self, Self::Generating { .. }));

        let mut n = GarblerStage::PreparedForEval { indexes_to_eval };

        mem::swap(self, &mut n);

        match n {
            Self::Generating { seeds } => seeds,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Garbler<I: CircuitInput + Clone> {
    stage: GarblerStage,
    instances: Vec<GarbledInstance>,
    config: Config<I>,
    live_capacity: usize,
    /// Nonce received from evaluator, stored for internal use in `commit_phase_two` and `do_soldering`
    nonce: Option<S>,
}

impl<I> Garbler<I>
where
    I: CircuitInput
        + Clone
        + Send
        + Sync
        + EncodeInput<GarbleMode<AesNiHasher, AESAccumulatingHash>>,
    <I as CircuitInput>::WireRepr: Send,
    I: 'static,
{
    /// Create garbled instances in parallel using the provided circuit builder function.
    pub fn create<F>(mut rng: impl Rng, config: Config<I>, live_capacity: usize, builder: F) -> Self
    where
        F: Fn(
                &mut StreamingMode<GarbleMode<AesNiHasher, AESAccumulatingHash>>,
                &I::WireRepr,
            ) -> WireId
            + Send
            + Sync
            + Copy,
    {
        let seeds = (0..config.total)
            .map(|_| rng.r#gen())
            .collect::<Box<[Seed]>>();

        // Use optimized thread pool internally
        let instances: Vec<_> = super::get_optimized_pool().install(|| {
            seeds
                .par_iter()
                .enumerate()
                .map(|(index, garbling_seed)| {
                    let inputs = config.input.clone();
                    let hasher = AESAccumulatingHash::default();

                    let span = tracing::info_span!("garble", instance = index);
                    let _enter = span.enter();

                    info!("Starting garbling of circuit (cut-and-choose)");

                    let res: StreamingResult<
                        GarbleMode<AesNiHasher, AESAccumulatingHash>,
                        I,
                        GarbledWire,
                    > = CircuitBuilder::streaming_garbling(
                        inputs,
                        live_capacity,
                        *garbling_seed,
                        hasher,
                        builder,
                    );

                    GarbledInstance::from(res)
                })
                .collect()
        });

        Self {
            stage: GarblerStage::Generating { seeds },
            instances,
            live_capacity,
            config,
            nonce: None,
        }
    }

    /// Produce the `Commit₁` transcript for every garbled instance (spec Step 1.2).
    pub fn commit_phase_one<HHasher>(&self) -> Vec<CommitPhaseOne<HHasher>>
    where
        HHasher: LabelCommitHasher,
    {
        self.instances
            .iter()
            .map(CommitPhaseOne::<HHasher>::from_instance)
            .collect()
    }

    /// Produce the `Commit₂` transcript (nonce-injected input commitments; spec Step 1.4).
    /// Stores the nonce internally for use in `do_soldering`.
    /// If called multiple times, the nonce must be the same; otherwise panics.
    pub fn commit_phase_two<HHasher>(&mut self, nonce: S) -> Vec<CommitPhaseTwo<HHasher>>
    where
        HHasher: LabelCommitHasher,
    {
        if let Some(existing_nonce) = self.nonce {
            if existing_nonce != nonce {
                panic!("Different nonce provided to commit_phase_two; nonce must be consistent");
            }
        } else {
            self.nonce = Some(nonce);
        }

        self.instances
            .iter()
            .map(|instance| CommitPhaseTwo::<HHasher>::from_instance(instance, self.nonce.unwrap()))
            .collect()
    }

    /// Get both phase one and phase two commitments (backward compatibility)
    pub fn get_commitment<HHasher: LabelCommitHasher>(&self) -> Option<super::Commitment<HHasher>> {
        self.nonce.map(|nonce| {
            let phase_one = self
                .instances
                .iter()
                .map(CommitPhaseOne::<HHasher>::from_instance)
                .collect();

            let phase_two = self
                .instances
                .iter()
                .map(|instance| CommitPhaseTwo::<HHasher>::from_instance(instance, nonce))
                .collect();

            (phase_one, phase_two)
        })
    }

    /// Get finalized indexes (backward compatibility)
    pub fn finalized_indexes(&self) -> Option<&[usize]> {
        match &self.stage {
            GarblerStage::Generating { .. } => None,
            GarblerStage::PreparedForEval { indexes_to_eval } => Some(indexes_to_eval),
        }
    }

    /// Open commitment without ciphertext handlers (backward compatibility)
    pub fn open_commit_without_ciphertexts(
        &mut self,
        mut indexes_to_finalize: Vec<usize>,
    ) -> OpenCommit {
        indexes_to_finalize.sort();
        indexes_to_finalize.dedup();

        assert_eq!(indexes_to_finalize.len(), self.config().to_finalize());

        let seeds = self
            .stage
            .next_stage(indexes_to_finalize.clone().into_boxed_slice());

        let mut commit = OpenCommit {
            open: vec![],
            closed: vec![],
        };

        seeds
            .into_vec()
            .into_iter()
            .enumerate()
            .for_each(|(index, seed)| {
                if indexes_to_finalize.binary_search(&index).is_ok() {
                    commit.closed.push((index, seed));
                } else {
                    commit.open.push((index, seed));
                }
            });

        commit
    }

    pub fn open_commit<F, CTH: 'static + Send + CiphertextHandler>(
        &mut self,
        mut indexes_to_finalize: Vec<(usize, CTH)>,
        builder: F,
    ) -> Vec<OpenForInstance>
    where
        F: 'static
            + Fn(&mut StreamingMode<GarbleMode<AesNiHasher, CTH>>, &I::WireRepr) -> WireId
            + Send
            + Sync
            + Copy,
        I: EncodeInput<GarbleMode<AesNiHasher, CTH>>,
    {
        let seeds = self
            .stage
            .next_stage(indexes_to_finalize.iter().map(|(i, _)| *i).collect());

        // TODO #37 Since at this point the number but finalization is no more than 7, we just run
        // threads here, without rayon
        seeds
            .iter()
            .enumerate()
            .map(|(index, garbling_seed)| {
                let pos = indexes_to_finalize
                    .iter()
                    .position(|(index_to_eval, _sender)| index_to_eval.eq(&index));

                if let Some(pos) = pos {
                    let sender = indexes_to_finalize.remove(pos).1;

                    let inputs = self.config.input.clone();
                    let garbling_seed = *garbling_seed;

                    let live_capacity = self.live_capacity;

                    let garbling_thread = thread::spawn(move || {
                        let _span =
                            tracing::info_span!("regarble2send", instance = index).entered();

                        info!("Starting");

                        let _: StreamingResult<_, I, GarbledWire> =
                            CircuitBuilder::<GarbleMode<AesNiHasher, _>>::streaming_garbling(
                                inputs,
                                live_capacity,
                                garbling_seed,
                                sender,
                                builder,
                            );
                    });

                    OpenForInstance::Closed {
                        index,
                        garbling_thread,
                    }
                } else {
                    OpenForInstance::Open(index, *garbling_seed)
                }
            })
            .collect()
    }

    #[cfg(feature = "sp1-soldering")]
    pub fn do_soldering(&self) -> SolderingProof {
        let nonce = self
            .nonce
            .expect("Nonce must be set before calling do_soldering");
        let GarblerStage::PreparedForEval { indexes_to_eval } = &self.stage else {
            panic!("Garbler not ready to soldering")
        };

        let mut indexes_to_eval = indexes_to_eval.clone();
        indexes_to_eval.sort();

        // Collect all instances (base + additional) into a single vector
        let mut all_instances = Vec::new();
        for &index in indexes_to_eval.iter() {
            all_instances.push(self.instances[index].input_wire_values.clone());
        }

        // Convert nonce from S to u128
        let nonce = nonce.to_u128();

        sp1_soldering::prove_soldering(all_instances, nonce)
    }

    /// Return the constant labels for true/false as u128 words for a given instance.
    pub fn true_wire_constant_for(&self, index: usize) -> u128 {
        self.instances[index]
            .true_wire_constant
            .select(true)
            .to_u128()
    }

    /// Return the constant labels for true/false as u128 words for a given instance.
    pub fn false_wire_constant_for(&self, index: usize) -> u128 {
        self.instances[index]
            .false_wire_constant
            .select(false)
            .to_u128()
    }

    /// Return a clone of the input garbled labels for a given instance.
    pub fn input_labels_for(&self, index: usize) -> Vec<GarbledWire> {
        self.instances[index].input_wire_values.clone()
    }

    pub fn config(&self) -> &Config<I> {
        &self.config
    }

    pub fn stage(&self) -> &GarblerStage {
        &self.stage
    }

    pub fn output_wire(&self, index: usize) -> Option<&GarbledWire> {
        self.instances.get(index).map(|gw| &gw.output_wire_values)
    }

    /// Returns input wire commitments for the base soldered instance.
    ///
    /// Available after `commit_phase_two` and finalized indexes are set (PreparedForEval stage).
    /// The base instance is the first finalized index.
    ///
    /// Returns `None` if not in PreparedForEval stage.
    pub fn soldered_base_commitment<H: LabelCommitHasher>(
        &self,
    ) -> Option<Vec<LabelCommit<H::Output>>> {
        let GarblerStage::PreparedForEval { indexes_to_eval } = &self.stage else {
            return None;
        };

        // Base instance is the first finalized index
        let base_idx = *indexes_to_eval.first()?;
        let base_instance = &self.instances[base_idx];

        Some(commit_input_wires::<H>(
            &base_instance.input_wire_values,
            self.nonce,
        ))
    }

    /// Returns output label commitments (true, false) for all finalized instances.
    ///
    /// Available after finalized indexes are set (PreparedForEval stage).
    /// Returns a vector of tuples where each tuple contains:
    /// - First element: commitment to the label when output is true
    /// - Second element: commitment to the label when output is false
    ///
    /// Returns `None` if not in PreparedForEval stage.
    pub fn finalized_output_label_commitment<H: LabelCommitHasher>(
        &self,
    ) -> Option<Vec<(H::Output, H::Output)>> {
        let GarblerStage::PreparedForEval { indexes_to_eval } = &self.stage else {
            return None;
        };

        Some(
            indexes_to_eval
                .iter()
                .map(|&idx| {
                    let instance = &self.instances[idx];
                    (
                        commit_output_label1::<H>(&instance.output_wire_values),
                        commit_output_label0::<H>(&instance.output_wire_values),
                    )
                })
                .collect(),
        )
    }
}

#[cfg(feature = "test-utils")]
mod test_utils {
    use super::*;
    use crate::circuit::CircuitInput;

    #[derive(Clone, Debug)]
    pub struct CommitPhaseOneRawParts<H: Clone + Copy> {
        pub ciphertext_hash: CiphertextCommit,
        pub input_commitments: Vec<LabelCommit<H>>,
        pub output_label1_commit: H,
        pub output_label0_commit: H,
        pub true_constant: u128,
        pub false_constant: u128,
    }

    impl<H: LabelCommitHasher> CommitPhaseOne<H> {
        /// Construct a commit payload directly from raw components for testing helpers.
        pub fn from_raw_parts(parts: CommitPhaseOneRawParts<H::Output>) -> Self {
            Self {
                ciphertext_hash: parts.ciphertext_hash,
                input_commitments: parts.input_commitments,
                output_label1_commit: parts.output_label1_commit,
                output_label0_commit: parts.output_label0_commit,
                true_constant: parts.true_constant,
                false_constant: parts.false_constant,
            }
        }

        pub fn into_raw_parts(self) -> CommitPhaseOneRawParts<H::Output> {
            CommitPhaseOneRawParts {
                ciphertext_hash: self.ciphertext_hash,
                input_commitments: self.input_commitments,
                output_label1_commit: self.output_label1_commit,
                output_label0_commit: self.output_label0_commit,
                true_constant: self.true_constant,
                false_constant: self.false_constant,
            }
        }
    }

    impl<H: LabelCommitHasher> CommitPhaseTwo<H> {
        pub fn from_raw_parts(input_commitments: Vec<LabelCommit<H::Output>>) -> Self {
            Self { input_commitments }
        }

        pub fn into_raw_parts(self) -> Vec<LabelCommit<H::Output>> {
            self.input_commitments
        }
    }

    impl<I> Garbler<I>
    where
        I: CircuitInput + Clone,
    {
        pub fn from_raw_parts(
            stage: GarblerStage,
            instances: Vec<GarbledInstance>,
            config: Config<I>,
            live_capacity: usize,
            nonce: Option<S>,
        ) -> Self {
            Self {
                stage,
                instances,
                config,
                live_capacity,
                nonce,
            }
        }

        pub fn into_raw_parts(
            self,
        ) -> (
            GarblerStage,
            Vec<GarbledInstance>,
            Config<I>,
            usize,
            Option<S>,
        ) {
            (
                self.stage,
                self.instances,
                self.config,
                self.live_capacity,
                self.nonce,
            )
        }
    }
}

#[cfg(feature = "test-utils")]
pub use test_utils::*;
