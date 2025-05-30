use crate::bag::*;
use crate::bristol::parser;
use rand::{Rng, rng};

pub struct Circuit {
    pub nog: usize,
    pub now: usize,
    pub input_sizes: Vec<usize>,
    pub output_sizes: Vec<usize>,
    pub wires: Vec<Rc<RefCell<Wire>>>,
    pub gates: Vec<Gate>,
}

impl Circuit {
    pub fn from_bristol(filename: &str) -> Self {
        parser(filename)
    }

    pub fn garbled_gates(&self) -> Vec<Vec<S>> {
        self.gates.iter().map(|gate| gate.garbled()).collect()
    }

    pub fn set_input_wires(&self) {
        for i in 0..self.input_sizes.iter().sum() {
            self.wires[i].borrow_mut().set(rng().random());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvm::bigint::U256;
    use bitvm::treepp::*;
    use std::iter::zip;

    fn test_circuit(circuit_filename: &str, correct: bool) {
        println!("testing {:?}", circuit_filename);
        let circuit = Circuit::from_bristol(circuit_filename);

        let mut garbled_gates = circuit.garbled_gates();
        let n = garbled_gates.len();

        if !correct {
            let u: u32 = rng().random();
            garbled_gates[(u as usize) % n] =
                vec![S::random(), S::random(), S::random(), S::random()];
        }

        circuit.set_input_wires();

        println!(
            "testing {:?} garble",
            if correct { "correct" } else { "incorrect" }
        );

        for (i, (gate, garble)) in zip(circuit.gates.clone(), garbled_gates).enumerate() {
            let a = gate.wire_a.borrow().get_label();
            let b = gate.wire_b.borrow().get_label();
            let bit_a = gate.wire_a.borrow().get_value();
            let bit_b = gate.wire_b.borrow().get_value();
            let bit_c = (gate.f())(bit_a, bit_b);
            let (garble_check, c) = gate.check_garble(garble.clone(), bit_c);
            let gate_script = gate.script(garble, garble_check);

            println!(
                "testing gate[{:?}], garble is {:?}",
                i,
                if garble_check { "correct" } else { "incorrect" }
            );

            let script = script! {
                { U256::push_hex(&hex::encode(&a.0)) }
                { if bit_a {1} else {0} }
                { U256::push_hex(&hex::encode(&b.0)) }
                { if bit_b {1} else {0} }
                { gate_script }
            };
            let result = execute_script(script);
            assert!(result.success);

            if garble_check {
                gate.wire_c.borrow_mut().set2(bit_c, c);
            } else {
                assert!(!correct);
                break;
            }
        }
    }

    fn test_circuit_find_incorrect(circuit_filename: &str, correct: bool) {
        println!("testing {:?}", circuit_filename);
        let circuit = Circuit::from_bristol(circuit_filename);

        let mut garbled_gates = circuit.garbled_gates();
        let n = garbled_gates.len();

        if !correct {
            let u: u32 = rng().random();
            garbled_gates[(u as usize) % n] =
                vec![S::random(), S::random(), S::random(), S::random()];
        }

        circuit.set_input_wires();

        println!(
            "testing {:?} garble",
            if correct { "correct" } else { "incorrect" }
        );

        for (i, (gate, garble)) in zip(circuit.gates.clone(), garbled_gates).enumerate() {
            let a = gate.wire_a.borrow().get_label();
            let b = gate.wire_b.borrow().get_label();
            let bit_a = gate.wire_a.borrow().get_value();
            let bit_b = gate.wire_b.borrow().get_value();
            let bit_c = (gate.f())(bit_a, bit_b);
            let (garble_check, c) = gate.check_garble(garble.clone(), bit_c);

            println!(
                "testing gate[{:?}], garble is {:?}",
                i,
                if garble_check { "correct" } else { "incorrect" }
            );

            if garble_check {
                gate.wire_c.borrow_mut().set2(bit_c, c);
                continue;
            }
            assert!(!correct);

            let gate_script = gate.script(garble, garble_check);

            let script = script! {
                { U256::push_hex(&hex::encode(&a.0)) }
                { if bit_a {1} else {0} }
                { U256::push_hex(&hex::encode(&b.0)) }
                { if bit_b {1} else {0} }
                { gate_script }
            };
            let result = execute_script(script);
            assert!(result.success);

            break;
        }
    }

    #[test]
    fn test_circuit_adder_find_incorrect() {
        test_circuit_find_incorrect("adder64.txt", true);
        test_circuit_find_incorrect("adder64.txt", false);
    }

    #[test]
    fn test_circuit_subtracter_find_incorrect() {
        test_circuit_find_incorrect("subtracter64.txt", true);
        test_circuit_find_incorrect("subtracter64.txt", false);
    }

    #[test]
    fn test_circuit_multiplier_find_incorrect() {
        test_circuit_find_incorrect("multiplier64.txt", true);
        test_circuit_find_incorrect("multiplier64.txt", false);
    }

    #[test]
    fn test_circuit_adder() {
        test_circuit("adder64.txt", true);
        test_circuit("adder64.txt", false);
    }

    #[test]
    fn test_circuit_subtracter() {
        test_circuit("subtracter64.txt", true);
        test_circuit("subtracter64.txt", false);
    }

    #[test]
    fn test_circuit_multiplier() {
        test_circuit("multiplier64.txt", true);
        test_circuit("multiplier64.txt", false);
    }
}
