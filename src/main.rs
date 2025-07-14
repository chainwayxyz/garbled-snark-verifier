use std::time::Instant;
use garbled_snark_verifier::{bag::{Circuit, Wires}, circuits::bn254::fq12::Fq12};
use rand::{rngs::StdRng, SeedableRng};
// use blake3::hash;

pub fn fq12_mul_equal(a: Wires, b: Wires, c: Wires) -> Circuit {
    let mut circuit = Circuit::empty();
    let d = circuit.extend(Fq12::mul_montgomery(a, b));
    let equality = circuit.extend(Fq12::equal(c, d));
    circuit.add_wire(equality[0].clone());
    circuit
}

pub fn main() {
    let mut r = StdRng::seed_from_u64(31);
    
    let start = Instant::now();
    let a = Fq12::random();
    let b = Fq12::random();
    let a_wires = Fq12::wires_set_rng(Fq12::as_montgomery(a), &mut r);
    let b_wires = Fq12::wires_set_rng(Fq12::as_montgomery(b), &mut r);
    let c_wires = Fq12::wires_set_rng(Fq12::as_montgomery(a * b), &mut r);
    let circuit = fq12_mul_equal(a_wires, b_wires, c_wires);
    circuit.gate_counts().print();
    println!("circuit generation: {:?}", start.elapsed());

    let start = Instant::now();
    circuit.evaluate();
    let result = circuit.0[0].clone();
    assert!(result.borrow().get_value());
    println!("circuit evaluation: {:?}", start.elapsed());

    let start = Instant::now();
    let garbles = circuit.garbled_gates();
    println!("garble len: {:?}", garbles.len());
    println!("circuit garbling: {:?}", start.elapsed());

    let start = Instant::now();
    circuit.garble_evaluate(garbles);
    assert_eq!(circuit.0[0].borrow().get_label(), circuit.0[0].borrow().select(true));
    println!("circuit garbling evaluation: {:?}", start.elapsed());
    println!("final label: {:?}", circuit.0[0].borrow().get_label());
}
