use super::BigIntImpl;
use crate::{bag::*, circuits::bigint::utils::bits_from_biguint};
use num_bigint::BigUint;

impl<const N_BITS: usize> BigIntImpl<N_BITS> {
    pub fn self_or_zero(
        input_wires: Vec<Rc<RefCell<Wire>>>,
    ) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), N_BITS + 1);
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut self_wires = input_wires.clone();
        let selector = self_wires.pop().unwrap();
        for i in 0..N_BITS {
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

    pub fn mul(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), N_BITS * 2);
        let mut a_wires = vec![];
        let mut b_wires = vec![];
        for i in 0..N_BITS {
            a_wires.push(input_wires[i].clone());
            b_wires.push(input_wires[i + N_BITS].clone());
        }

        let mut result: Vec<Rc<RefCell<Wire>>> = vec![];
        let mut circuit_gates = vec![];
        for _ in 0..(N_BITS * 2) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(false);
            result.push(wire)
        }
        for i in 0..N_BITS {
            let mut aux = a_wires.clone();
            aux.push(b_wires[i].clone());
            let (mut addition_wires, gates) = Self::self_or_zero(aux);
            circuit_gates.extend(gates);
            for j in i..(i + N_BITS) {
                addition_wires.push(result[j].clone());
            }
            let (new_bits, gates) = Self::add(addition_wires);
            circuit_gates.extend(gates);
            for j in i..=(i + N_BITS) {
                result[j] = new_bits[j - i].clone();
            }
        }
        return (result, circuit_gates);
    }

    ///Assuming constant is smaller than 2^N_BITS, and returns 2 * N_BITS result for now (can be optimized)
    /// Instead of using bits, this can also be further optimized with negative and positive bits (similar to ate pairing loop thingy (Fatih mentioned, :speaking_head_in_silhouette:))
    pub fn mul_by_constant(
        input_wires: Vec<Rc<RefCell<Wire>>>,
        c: BigUint,
    ) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!(input_wires.len(), N_BITS);
        let mut c_bits = bits_from_biguint(c);
        c_bits.truncate(N_BITS);
        //assert!(c_bits.len() < N_BITS, "{} is too long", c_bits.len());
        //this check doesn't work for now

        let mut a_wires = vec![];
        for i in 0..N_BITS {
            a_wires.push(input_wires[i].clone());
        }

        let mut result: Vec<Rc<RefCell<Wire>>> = vec![];
        let mut circuit_gates = vec![];
        for _ in 0..(N_BITS * 2) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(false);
            result.push(wire)
        }
        for (i, bit) in c_bits.iter().enumerate() {
            if *bit {
                let mut addition_wires = a_wires.clone();
                for j in i..(i + N_BITS) {
                    addition_wires.push(result[j].clone());
                }
                let (new_bits, gates) = Self::add(addition_wires);
                circuit_gates.extend(gates);
                for j in i..=(i + N_BITS) {
                    result[j] = new_bits[j - i].clone();
                }
            }
        }
        return (result, circuit_gates);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::bigint::{U254, utils::biguint_from_bits};
    use rand::{Rng, rng};

    //tests are currently only for 254 bits

    #[test]
    fn test_mul() {
        for _ in 0..10 {
            let mut input_wires = Vec::new();
            let mut bits1 = Vec::new();
            let mut bits2 = Vec::new();
            for _ in 0..U254::N_BITS {
                let bit = rng().random::<bool>();
                let new_wire = Rc::new(RefCell::new(Wire::new()));
                new_wire.borrow_mut().set(bit);
                input_wires.push(new_wire);
                bits1.push(bit);
            }
            for _ in 0..U254::N_BITS {
                let bit = rng().random::<bool>();
                let new_wire = Rc::new(RefCell::new(Wire::new()));
                new_wire.borrow_mut().set(bit);
                input_wires.push(new_wire);
                bits2.push(bit);
            }

            let a = biguint_from_bits(bits1);
            let b = biguint_from_bits(bits2);
            let c = a * b;
            let (output_wires, gates) = U254::mul(input_wires);
            println!("gate count: {:?}", gates.len());

            for mut gate in gates.clone() {
                gate.evaluate();
            }

            let result = biguint_from_bits(
                output_wires
                    .iter()
                    .map(|output_wire| output_wire.borrow().get_value())
                    .collect(),
            );
            assert_eq!(result, c);
        }
    }

    #[test]
    fn test_mul_by_constant() {
        for _ in 0..10 {
            let mut input_wires = Vec::new();
            let mut bits1 = Vec::new();
            for _ in 0..U254::N_BITS {
                let bit = rng().random::<bool>();
                let new_wire = Rc::new(RefCell::new(Wire::new()));
                new_wire.borrow_mut().set(bit);
                input_wires.push(new_wire);
                bits1.push(bit);
            }

            let a = biguint_from_bits(bits1);
            let b = BigUint::from_bytes_le(&rng().random::<[u8; 31]>()); //Not the exact bound, but should be fine for now, fix later
            let c = a * b.clone();
            let (output_wires, gates) = U254::mul_by_constant(input_wires, b);
            println!("gate count: {:?}", gates.len());

            for mut gate in gates.clone() {
                gate.evaluate();
            }

            let result = biguint_from_bits(
                output_wires
                    .iter()
                    .map(|output_wire| output_wire.borrow().get_value())
                    .collect(),
            );
            assert_eq!(result, c);
        }
    }
}
