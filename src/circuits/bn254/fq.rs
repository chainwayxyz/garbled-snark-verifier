use super::Fp254Impl;
use crate::{
    bag::*,
    circuits::{
        bigint::{BigIntImpl, U254, utils::bits_from_biguint},
        bn254::utils::bits_from_fq,
    },
};
use num_bigint::BigUint;
use std::str::FromStr;
pub struct Fq;

impl Fp254Impl for Fq {
    const MODULUS: &'static str =
        "21888242871839275222246405745257275088696311157297823662689037894645226208583";
}

impl Fq {
    pub fn modulus_as_biguint() -> BigUint {
        BigUint::from_str(Self::MODULUS).unwrap()
    }

    pub fn modulus_as_bits() -> Vec<bool> {
        bits_from_biguint(Fq::modulus_as_biguint())
    }

    pub fn not_modulus_as_biguint() -> BigUint {
        let p = Fq::modulus_as_biguint();
        let a = BigUint::from_str("2")
            .unwrap()
            .pow(U254::N_BITS.try_into().unwrap());
        a - p
    }

    pub fn not_modulus_as_bits() -> Vec<bool> {
        bits_from_biguint(Fq::not_modulus_as_biguint())
    }

    pub fn self_or_zero(
        input_wires: Vec<Rc<RefCell<Wire>>>,
    ) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS + 1);
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut self_wires = input_wires.clone();
        let selector = self_wires.pop().unwrap();
        for i in 0..U254::N_BITS {
            let wire = Rc::new(RefCell::new(Wire::new()));
            let gate = Gate::new(
                self_wires[i].clone(),
                selector.clone(),
                wire.clone(),
                "and".to_string(),
            );
            circuit_gates.push(gate);
            output_wires.push(wire);
        }
        (output_wires, circuit_gates)
    }

    pub fn add(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), 2 * U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let (mut wires_1, gates_1) = U254::add(input_wires);
        let u = wires_1.pop().unwrap();
        let c = Fq::not_modulus_as_biguint();
        let (mut wires_2, gates_2) = U254::add_constant(wires_1.clone(), c);
        wires_2.pop();
        let not_u = Rc::new(RefCell::new(Wire::new()));
        let gate_3 = Gate::new(u.clone(), u.clone(), not_u.clone(), "inv".to_string());
        let (v, gates_4) = U254::less_than_constant(wires_1.clone(), Fq::modulus_as_biguint());
        let selector = Rc::new(RefCell::new(Wire::new()));
        let gate_5 = Gate::new(
            not_u.clone(),
            v[0].clone(),
            selector.clone(),
            "and".to_string(),
        );
        wires_1.extend(wires_2);
        wires_1.push(selector);
        let (output_wires, gates_6) = U254::select(wires_1);
        circuit_gates.extend(gates_1);
        circuit_gates.extend(gates_2);
        circuit_gates.push(gate_3);
        circuit_gates.extend(gates_4);
        circuit_gates.push(gate_5);
        circuit_gates.extend(gates_6);
        (output_wires, circuit_gates)
    }

    pub fn sub(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), 2 * U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in 0..U254::N_BITS * 2 {
            if i < U254::N_BITS {
                a.push(input_wires[i].clone());
            } else {
                b.push(input_wires[i].clone());
            }
        }
        let (wires_1, gates_1) = Fq::neg(b);
        a.extend(wires_1);
        let (output_wires, gates_2) = Fq::add(a);
        circuit_gates.extend(gates_1);
        circuit_gates.extend(gates_2);

        (output_wires, circuit_gates)
    }

    pub fn add_constant(
        input_wires: Vec<Rc<RefCell<Wire>>>,
        b: ark_bn254::Fq,
    ) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let (mut wires_1, gates_1) = U254::add_constant(input_wires, BigUint::from(b));
        let u = wires_1.pop().unwrap();
        let c = Fq::not_modulus_as_biguint();
        let (mut wires_2, gates_2) = U254::add_constant(wires_1.clone(), c);
        wires_2.pop();
        let not_u = Rc::new(RefCell::new(Wire::new()));
        let gate_3 = Gate::new(u.clone(), u.clone(), not_u.clone(), "inv".to_string());
        let (v, gates_4) = U254::less_than_constant(wires_1.clone(), Fq::modulus_as_biguint());
        let selector = Rc::new(RefCell::new(Wire::new()));
        let gate_5 = Gate::new(
            not_u.clone(),
            v[0].clone(),
            selector.clone(),
            "and".to_string(),
        );
        wires_1.extend(wires_2);
        wires_1.push(selector);
        let (output_wires, gates_6) = U254::select(wires_1);
        circuit_gates.extend(gates_1);
        circuit_gates.extend(gates_2);
        circuit_gates.push(gate_3);
        circuit_gates.extend(gates_4);
        circuit_gates.push(gate_5);
        circuit_gates.extend(gates_6);
        (output_wires, circuit_gates)
    }

    pub fn neg(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let mut not_wires = Vec::new();
        for i in 0..U254::N_BITS {
            let not_wire = Rc::new(RefCell::new(Wire::new()));
            let gate = Gate::new(
                input_wires[i].clone(),
                input_wires[i].clone(),
                not_wire.clone(),
                "not".to_string(),
            );
            not_wires.push(not_wire);
            circuit_gates.push(gate);
        }

        let (output_wires, gates) = Fq::add_constant(
            not_wires,
            ark_bn254::Fq::from(1) - ark_bn254::Fq::from(Fq::not_modulus_as_biguint()),
        );
        circuit_gates.extend(gates);
        (output_wires, circuit_gates)
    }

    pub fn double(mut input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let shift_wire = Rc::new(RefCell::new(Wire::new()));
        shift_wire.borrow_mut().set(false);
        let u = input_wires.pop().unwrap();
        let mut shifted_wires = vec![shift_wire];
        shifted_wires.extend(input_wires);
        let c = Fq::not_modulus_as_biguint();
        let (mut wires_2, gates_1) = U254::add_constant(shifted_wires.clone(), c);
        wires_2.pop();
        let not_u = Rc::new(RefCell::new(Wire::new()));
        let gate_3 = Gate::new(u.clone(), u.clone(), not_u.clone(), "inv".to_string());
        let (v, gates_2) =
            U254::less_than_constant(shifted_wires.clone(), Fq::modulus_as_biguint());
        let selector = Rc::new(RefCell::new(Wire::new()));
        let gate_4 = Gate::new(
            not_u.clone(),
            v[0].clone(),
            selector.clone(),
            "and".to_string(),
        );
        shifted_wires.extend(wires_2);
        shifted_wires.push(selector);
        let (output_wires, gates_5) = U254::select(shifted_wires);
        circuit_gates.extend(gates_1);
        circuit_gates.extend(gates_2);
        circuit_gates.push(gate_3);
        circuit_gates.push(gate_4);
        circuit_gates.extend(gates_5);
        (output_wires, circuit_gates)
    }

    pub fn mul(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), 2 * U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let mut a_wires = Vec::new();
        let mut b_wires = Vec::new();
        for i in 0..U254::N_BITS {
            a_wires.push(input_wires[i].clone());
            b_wires.push(input_wires[i + U254::N_BITS].clone());
        }
        let mut a_wires_with_selector = a_wires.clone();
        a_wires_with_selector.push(b_wires[U254::N_BITS - 1].clone());
        let (a_or_zero_wires, a_or_zero_gates) = Fq::self_or_zero(a_wires_with_selector);
        circuit_gates.extend(a_or_zero_gates);
        let mut result = a_or_zero_wires.clone();
        for b_wire in b_wires.iter().rev().skip(1) {
            let (double_wires, double_gates) = Fq::double(result.clone());
            circuit_gates.extend(double_gates);
            let mut a_wires_with_selector_i = a_wires.clone();
            a_wires_with_selector_i.push(b_wire.clone());
            let (a_or_zero_wires_i, a_or_zero_gates_i) = Fq::self_or_zero(a_wires_with_selector_i);
            circuit_gates.extend(a_or_zero_gates_i);
            let mut add_inputs = double_wires.clone();
            add_inputs.extend(a_or_zero_wires_i.clone());
            let (add_wires, add_gates) = Fq::add(add_inputs);
            circuit_gates.extend(add_gates);
            result = add_wires;
        }
        (result, circuit_gates)
    }

    pub fn mul_by_constant(
        input_wires: Vec<Rc<RefCell<Wire>>>,
        b: ark_bn254::Fq,
    ) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let mut a_wires = Vec::new();
        for i in 0..U254::N_BITS {
            a_wires.push(input_wires[i].clone());
        }
        let b_bits = bits_from_fq(b);
        let mut i = U254::N_BITS - 1;
        while !b_bits[i] {
            i -= 1;
        }
        let mut result = a_wires.clone();
        for b_bit in b_bits.iter().rev().skip(U254::N_BITS - i) {
            let (double_wires, double_gates) = Fq::double(result.clone());
            circuit_gates.extend(double_gates);
            if *b_bit {
                let mut add_inputs = double_wires.clone();
                add_inputs.extend(a_wires.clone());
                let (add_wires, add_gates) = Fq::add(add_inputs);
                circuit_gates.extend(add_gates);
                result = add_wires;
            } else {
                result = double_wires;
            }
        }
        (result, circuit_gates)
    }

    pub fn square(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS);
        let mut circuit_gates = Vec::new();
        let mut a_wires = Vec::new();
        for i in 0..U254::N_BITS {
            a_wires.push(input_wires[i].clone());
        }
        let mut a_wires_with_selector = a_wires.clone();
        a_wires_with_selector.push(a_wires[U254::N_BITS - 1].clone());
        let (a_or_zero_wires, a_or_zero_gates) = Fq::self_or_zero(a_wires_with_selector);
        circuit_gates.extend(a_or_zero_gates);
        let mut result = a_or_zero_wires.clone();
        for a_wire in a_wires.iter().rev().skip(1) {
            let (double_wires, double_gates) = Fq::double(result.clone());
            circuit_gates.extend(double_gates);
            let mut a_wires_with_selector_i = a_wires.clone();
            a_wires_with_selector_i.push(a_wire.clone());
            let (a_or_zero_wires_i, a_or_zero_gates_i) = Fq::self_or_zero(a_wires_with_selector_i);
            circuit_gates.extend(a_or_zero_gates_i);
            let mut add_inputs = double_wires.clone();
            add_inputs.extend(a_or_zero_wires_i.clone());
            let (add_wires, add_gates) = Fq::add(add_inputs);
            circuit_gates.extend(add_gates);
            result = add_wires;
        }
        (result, circuit_gates)
    }

    pub fn hinted_mul(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), U254::N_BITS * 3);
        let mut a_wires = vec![];
        let mut b_wires = vec![];
        let mut hint_wires = vec![];
        for i in 0..U254::N_BITS {
            a_wires.push(input_wires[i].clone());
            b_wires.push(input_wires[i + U254::N_BITS].clone());
            hint_wires.push(input_wires[i + 2 * U254::N_BITS].clone());
        }
        let mut circuit_gates = vec![];

        let (r_wires, gates) = U254::mul(a_wires.iter().chain(b_wires.iter()).cloned().collect());
        circuit_gates.extend(gates);

        let (sub_wires, gates) =
            U254::mul_by_constant(hint_wires.clone(), Self::modulus_as_biguint());
        circuit_gates.extend(gates);

        const TWOTIMES: usize = 508; //bad to have a constant but there's nothing we can do 
        assert_eq!(r_wires.len(), TWOTIMES);
        assert_eq!(sub_wires.len(), TWOTIMES);
        type U508 = BigIntImpl<TWOTIMES>;

        let ((mut result_wires, gates), bound_check_1_wire) =
            U508::sub(r_wires.iter().chain(sub_wires.iter()).cloned().collect());
        circuit_gates.extend(gates);

        let (bound_check_2_wire, gates) =
            U508::less_than_constant(result_wires.clone(), Self::modulus_as_biguint());
        circuit_gates.extend(gates);

        let bound_checks = Rc::new(RefCell::new(Wire::new()));
        circuit_gates.push(Gate::new(
            bound_check_1_wire.clone(),
            bound_check_2_wire[0].clone(),
            bound_checks.clone(),
            "and".to_string(),
        ));

        result_wires.truncate(U254::N_BITS);
        result_wires.push(bound_checks);

        return (result_wires, circuit_gates);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::bn254::utils::{bits_from_fq, fq_from_bits, random_fq};

    #[test]
    fn test_fq_add() {
        let a = random_fq();
        let b = random_fq();
        let c = a + b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        for bit in bits_from_fq(b) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::add(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq_sub() {
        let a = random_fq();
        let b = random_fq();
        let c = a - b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        for bit in bits_from_fq(b) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::sub(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq_add_constant() {
        let a = random_fq();
        let b = random_fq();
        let c = a + b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::add_constant(input_wires, b);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_neg() {
        let a = random_fq();
        let c = -a;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::neg(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq_double() {
        let a = random_fq();
        let c = a + a;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::double(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq_mul() {
        let a = random_fq();
        let b = random_fq();
        let c = a * b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        for bit in bits_from_fq(b) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::mul(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq_hinted_mul() {
        for _ in 0..1 {
            let a = random_fq();
            let b = random_fq();
            let c = a * b;
            let no_modulo_mul = BigUint::from(a) * BigUint::from(b);
            let hint = no_modulo_mul.clone() / Fq::modulus_as_biguint();
            assert_eq!(
                no_modulo_mul - hint.clone() * Fq::modulus_as_biguint(),
                BigUint::from(c)
            );

            let mut input_wires = Vec::new();
            for bit in bits_from_fq(a) {
                let wire = Rc::new(RefCell::new(Wire::new()));
                wire.borrow_mut().set(bit);
                input_wires.push(wire)
            }

            for bit in bits_from_fq(b) {
                let wire = Rc::new(RefCell::new(Wire::new()));
                wire.borrow_mut().set(bit);
                input_wires.push(wire)
            }

            let mut hint_bits = bits_from_biguint(hint);
            hint_bits.resize(U254::N_BITS, false);
            for bit in hint_bits {
                let wire = Rc::new(RefCell::new(Wire::new()));
                wire.borrow_mut().set(bit);
                input_wires.push(wire)
            }

            let (mut output_wires, gates) = Fq::hinted_mul(input_wires);

            println!("gate count: {:?}", gates.len());
            for mut gate in gates {
                gate.evaluate();
            }
            let right_hint = output_wires.pop().unwrap();
            let d = fq_from_bits(
                output_wires
                    .iter()
                    .map(|output_wire| output_wire.borrow().get_value())
                    .collect(),
            );
            assert_eq!(right_hint.borrow().get_value(), true);
            assert_eq!(c, d);
        }
    }

    #[test]
    #[should_panic(expected = "Wrong hint")]
    fn test_fq_hinted_mul_faulty() {
        for _ in 0..1 {
            let a = random_fq();
            let b = random_fq();
            //let c = a * b;
            let no_modulo_mul = BigUint::from(a) * BigUint::from(b);
            let mut hint = no_modulo_mul / Fq::modulus_as_biguint();
            hint += BigUint::from_str("1").unwrap(); //lame tests but can be made better later 

            let mut input_wires = Vec::new();
            for bit in bits_from_fq(a) {
                let wire = Rc::new(RefCell::new(Wire::new()));
                wire.borrow_mut().set(bit);
                input_wires.push(wire)
            }

            for bit in bits_from_fq(b) {
                let wire = Rc::new(RefCell::new(Wire::new()));
                wire.borrow_mut().set(bit);
                input_wires.push(wire)
            }
            let mut hint_bits = bits_from_biguint(hint);
            hint_bits.resize(U254::N_BITS, false);
            for bit in hint_bits {
                let wire = Rc::new(RefCell::new(Wire::new()));
                wire.borrow_mut().set(bit);
                input_wires.push(wire)
            }

            let (mut output_wires, gates) = Fq::hinted_mul(input_wires);

            println!("gate count: {:?}", gates.len());
            for mut gate in gates {
                gate.evaluate();
            }
            let right_hint = output_wires.pop().unwrap();
            //let d = fq_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
            assert_eq!(right_hint.borrow().get_value(), true, "Wrong hint");
        }
    }

    #[test]
    fn test_fq_mul_by_constant() {
        let a = random_fq();
        let b = random_fq();
        let c = a * b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::mul_by_constant(input_wires, b);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq_square() {
        let a = random_fq();
        let c = a * a;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq::square(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq_from_bits(
            output_wires
                .iter()
                .map(|output_wire| output_wire.borrow().get_value())
                .collect(),
        );
        assert_eq!(c, d);
    }
}
