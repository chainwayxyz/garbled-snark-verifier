use super::BigIntImpl;
use crate::bag::*;

impl<const N_BITS: usize> BigIntImpl<N_BITS> {
    // Return ((result_cells, gates), bound_checks) (bound checks are false if subtracted number is bigger)
    pub fn sub(
        input_wires: Vec<Rc<RefCell<Wire>>>,
    ) -> ((Vec<Rc<RefCell<Wire>>>, Vec<Gate>), Rc<RefCell<Wire>>) {
        assert_eq!(input_wires.len(), 2 * N_BITS);
        let mut a_wires = vec![];
        let mut b_wires = vec![];
        for i in 0..N_BITS {
            a_wires.push(input_wires[i].clone());
            b_wires.push(input_wires[i + N_BITS].clone());
        }

        let mut circuit_gates = vec![];

        let mut result = vec![];

        let mut want: Rc<RefCell<Wire>> = Rc::new(RefCell::new(Wire::new()));
        for i in 0..N_BITS {
            result.push(Rc::new(RefCell::new(Wire::new())));
            if i > 0 {
                let subtract_bit = Rc::new(RefCell::new(Wire::new()));
                circuit_gates.push(Gate::new(
                    want.clone(),
                    b_wires[i].clone(),
                    subtract_bit.clone(),
                    "xor".to_string(),
                ));
                circuit_gates.push(Gate::new(
                    subtract_bit.clone(),
                    a_wires[i].clone(),
                    result[i].clone(),
                    "xor".to_string(),
                ));
                let new_want_or0 = Rc::new(RefCell::new(Wire::new()));
                let new_want_or1 = Rc::new(RefCell::new(Wire::new()));
                let new_want = Rc::new(RefCell::new(Wire::new()));
                circuit_gates.push(Gate::new(
                    subtract_bit.clone(),
                    a_wires[i].clone(),
                    new_want_or0.clone(),
                    "nimp".to_string(),
                ));
                circuit_gates.push(Gate::new(
                    want.clone(),
                    b_wires[i].clone(),
                    new_want_or1.clone(),
                    "and".to_string(),
                ));
                circuit_gates.push(Gate::new(
                    new_want_or0.clone(),
                    new_want_or1.clone(),
                    new_want.clone(),
                    "or".to_string(),
                ));
                want = new_want;
            } else {
                circuit_gates.push(Gate::new(
                    b_wires[i].clone(),
                    a_wires[i].clone(),
                    result[i].clone(),
                    "xor".to_string(),
                ));
                let new_want = Rc::new(RefCell::new(Wire::new()));
                circuit_gates.push(Gate::new(
                    b_wires[i].clone(),
                    a_wires[i].clone(),
                    new_want.clone(),
                    "nimp".to_string(),
                ));
                want = new_want;
            }
        }

        let bound_check_wire = Rc::new(RefCell::new(Wire::new()));
        circuit_gates.push(Gate::new(
            want.clone(),
            want.clone(),
            bound_check_wire.clone(),
            "not".to_string(),
        ));

        return ((result, circuit_gates), bound_check_wire);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::bigint::{U254, utils::biguint_from_bits};
    use rand::{Rng, rng};

    #[test]
    fn test_sub() {
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
            let ((output_wires, gates), bound_check) = U254::sub(input_wires);
            for mut gate in gates {
                gate.evaluate();
            }
            if a < b {
                assert_eq!(bound_check.borrow().get_value(), false);
            } else {
                assert_eq!(bound_check.borrow().get_value(), true);
                let result = biguint_from_bits(
                    output_wires
                        .iter()
                        .map(|output_wire| output_wire.borrow().get_value())
                        .collect(),
                );
                assert_eq!(result, a - b);
            }
        }
    }
}
