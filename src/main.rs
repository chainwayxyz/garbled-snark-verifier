use std::time::Instant;
use garbled_snark_verifier::{bag::{Circuit, Wires, S}, circuits::bn254::fq12::Fq12};
use rand::{rng, rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use blake3::hash;

pub fn custom_circuit(a: Wires, b: Wires, c: Wires) -> Circuit {
    let mut circuit = Circuit::empty();
    let d = circuit.extend(Fq12::mul_montgomery(a.clone(), b.clone()));
    let equality = circuit.extend(Fq12::equal(c, d));
    circuit.add_wire(equality[0].clone());
    circuit
}

pub fn gc_commitment(circuit: &Circuit) -> [u8; 32] {
    let garble = circuit.garbled_gates();
    let mut v = Vec::new();
    for (x, y) in garble.clone() {
        if x.is_some() {
            v.extend(x.unwrap().0);
            v.extend(y.unwrap().0);
        }
    }
    let temp = hash(&v);
    let garble_hash = temp.as_bytes();

    *garble_hash
}
 
pub fn garbler1<const N: usize, const M: usize>() -> ([[u8; 32]; N], ([u64; N], [Vec<(Option<S>, Option<S>)>; N])) {
    let mut commitments = [[0; 32]; N];
    let mut seeds = [0; N];
    let mut garbles = [const { Vec::new() }; N];
    for i in 0..N {
        let seed = rng().random();
        let mut r = StdRng::seed_from_u64(seed);
        let a_wires = Fq12::wires_set_labels_rng(&mut r);
        let b_wires = Fq12::wires_set_labels_rng(&mut r);
        let c_wires = Fq12::wires_set_labels_rng(&mut r);
        let circuit = custom_circuit(a_wires, b_wires, c_wires);
        let garble = circuit.garbled_gates();
        commitments[i] = gc_commitment(&circuit);
        seeds[i] = seed;
        garbles[i] = garble;
    }
    (commitments, (seeds, garbles))
}

pub fn evaluator1<const N: usize, const M: usize>() -> [usize; M] {
    let mut rng = rng();
    let mut v = [0_usize; M];
    let choice = (0..N).choose_multiple(&mut rng, M);
    for i in 0..M {
        v[i] = choice[i];
    }
    v
}

pub fn garbler2<const N: usize, const M: usize, const N_MINUS_M: usize>(selected: [usize; M], seeds: [u64; N], garbles: [Vec<(Option<S>, Option<S>)>; N]) -> ([Vec<(Option<S>, Option<S>)>; M], [u64; N_MINUS_M]) {
    assert_eq!(N_MINUS_M, N - M);
    let mut selected_garbles = [const { Vec::new() }; M];
    let mut opened_seeds = [0_u64; N_MINUS_M];
    let mut selected_index = 0;
    let mut seed_index = 0;
    for i in 0..N {
        if selected.contains(&i) {
            selected_garbles[selected_index] = garbles[i].clone();
            selected_index += 1;
        }
        else {
            opened_seeds[seed_index] = seeds[i].clone();
            seed_index += 1;
        }
    }
    (selected_garbles, opened_seeds)
}

pub fn evaluator2<const N: usize, const M: usize, const N_MINUS_M: usize>(commitments: [[u8; 32]; N], selected: [usize; M], opened_seeds: [u64; N_MINUS_M]) {
    let mut seed_index = 0;
    for i in 0..N {
        if !selected.contains(&i) {
            let seed = opened_seeds[seed_index];
            let commitment = commitments[i];
            let mut r = StdRng::seed_from_u64(seed);
            let a_wires = Fq12::wires_set_labels_rng(&mut r);
            let b_wires = Fq12::wires_set_labels_rng(&mut r);
            let c_wires = Fq12::wires_set_labels_rng(&mut r);
            let circuit = custom_circuit(a_wires, b_wires, c_wires);
            let garble_hash = gc_commitment(&circuit);
            assert_eq!(garble_hash, commitment);
            seed_index += 1;
        }
    }
}

pub fn garbler3() {
    // .
}

pub fn evaluator3() {
    // .
}

pub fn garbler() {
    const N: usize = 10;
    const M: usize = 3;
    const N_MINUS_M: usize = 7;
    let (commitments, (seeds, garbles)) = garbler1::<N, M>();
    for commitment in commitments {
        println!("commitment: {:?}", commitment);
    }
    let selected = [0, 0, 0]; // take from evalautor
    let (_selected_garbles, opened_seeds) = garbler2::<N, M, N_MINUS_M>(selected, seeds, garbles);
    for opened_seed in opened_seeds {
        println!("seed: {:?}", opened_seed);
    }
}

pub fn evaluator() {
    const N: usize = 10;
    const M: usize = 3;
    const N_MINUS_M: usize = 7;
    let commitments = [
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
        [0_u8; 32],
    ]; // take from garbler
    let selected = evaluator1::<N, M>();
    println!("selected: {:?}", selected);
    let opened_seeds = [0, 0, 0, 0, 0, 0, 0];
    evaluator2::<N, M, N_MINUS_M>(commitments, selected, opened_seeds);
}

pub fn test() {
    let mut r = StdRng::seed_from_u64(31);
    
    let start = Instant::now();
    let a = Fq12::random();
    let b = Fq12::random();
    let a_wires = Fq12::wires_set_rng(Fq12::as_montgomery(a), &mut r);
    let b_wires = Fq12::wires_set_rng(Fq12::as_montgomery(b), &mut r);
    let c_wires = Fq12::wires_set_rng(Fq12::as_montgomery(a * b), &mut r);
    let circuit = custom_circuit(a_wires, b_wires, c_wires);
    circuit.gate_counts().print();
    println!("circuit generation: {:?}", start.elapsed());

    let start = Instant::now();
    circuit.evaluate();
    let result = circuit.0[0].clone();
    assert!(result.borrow().get_value());
    println!("circuit evaluation: {:?}", start.elapsed());

    let start = Instant::now();
    let garble = circuit.garbled_gates();
    let garble_hash = gc_commitment(&circuit);
    println!("garble_hash: {:?}", garble_hash);
    println!("circuit garbling: {:?}", start.elapsed());

    let start = Instant::now();
    circuit.garble_evaluate(garble);
    assert_eq!(circuit.0[0].borrow().get_label(), circuit.0[0].borrow().select(true));
    println!("circuit garbling evaluation: {:?}", start.elapsed());
    println!("final label: {:?}", circuit.0[0].borrow().get_label());
}

pub fn main() {
    test();
}
