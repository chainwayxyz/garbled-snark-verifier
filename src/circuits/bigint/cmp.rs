use num_bigint::BigUint;
use crate::circuits::bigint::utils::{bits_from_biguint, wires_for_u254};
use crate::circuits::bigint::U254;
use crate::{bag::*, circuits::basic::selector};
use super::BigIntImpl;

impl<const N_BITS: usize> BigIntImpl<N_BITS> {
    pub fn equal(a: Wires, b: Wires) -> Circuit {
        assert_eq!(a.len(), Self::N_BITS);
        assert_eq!(b.len(), Self::N_BITS);
        let mut circuit = Circuit::empty();

        let c = wires_for_u254();
        for i in 0..N_BITS {
            circuit.add(Gate::xor(a[i].clone(), b[i].clone(), c[i].clone()));
        }
        let result = circuit.extend(U254::equal_constant(c, BigUint::ZERO));
        circuit.add_wires(result);
        circuit
    }

    pub fn equal_constant(a: Wires, b: BigUint) -> Circuit {
        assert_eq!(a.len(), Self::N_BITS);
        let mut circuit = Circuit::empty();

        let b_bits = bits_from_biguint(b);
        let mut output = a[0].clone();
        if !b_bits[0] {
            let not_a0 = Rc::new(RefCell::new(Wire::new()));
            circuit.add(Gate::not(a[0].clone(), not_a0.clone()));
            output = not_a0;
        }

        for i in 1..N_BITS {
            let mut a_or_a_not = a[i].clone();
            if !b_bits[i] {
                let not_ai = Rc::new(RefCell::new(Wire::new()));
                circuit.add(Gate::not(a[i].clone(), not_ai.clone()));
                a_or_a_not = not_ai;
            }
            let new_output = Rc::new(RefCell::new(Wire::new()));
            circuit.add(Gate::and(output.clone(), a_or_a_not.clone(), new_output.clone()));
            output = new_output;
        }
        circuit.add_wire(output);
        circuit
    }

    pub fn greater_than(a: Wires, b: Wires) -> Circuit {
        assert_eq!(a.len(), N_BITS);
        assert_eq!(b.len(), N_BITS);
        let mut circuit = Circuit::empty();

        let not_b = wires_for_u254();

        for i in 0..N_BITS {
            circuit.add(Gate::not(b[i].clone(), not_b[i].clone()));
        }

        let wires = circuit.extend(U254::add(a, not_b));
        circuit.add_wire(wires[N_BITS].clone());
        circuit
    }

    pub fn less_than_constant(a: Wires, b: BigUint) -> Circuit {
        assert_eq!(a.len(), N_BITS);
        let mut circuit = Circuit::empty();

        let not_a = wires_for_u254();

        for i in 0..N_BITS {
            circuit.add(Gate::not(a[i].clone(), not_a[i].clone()));
        }

        let wires = circuit.extend(U254::add_constant(not_a, b));
        circuit.add_wire(wires[N_BITS].clone());
        circuit
    }

    pub fn select(a: Wires, b: Wires, s: Wirex) -> Circuit {
        assert_eq!(a.len(), N_BITS);
        assert_eq!(b.len(), N_BITS);
        let mut circuit = Circuit::empty();
        
        for i in 0..N_BITS {
            let wires = circuit.extend(selector(a[i].clone(), b[i].clone(), s.clone()));
            circuit.add_wire(wires[0].clone());
        }
        circuit
    }

    pub fn self_or_zero(a: Wires, s: Wirex) -> Circuit {
        assert_eq!(a.len(), Self::N_BITS);
        let mut circuit = Circuit::empty();

        let result = wires_for_u254();
        for i in 0..Self::N_BITS {
            circuit.add(Gate::and(a[i].clone(), s.clone(), result[i].clone()));
        }
        circuit.add_wires(result);
        circuit
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::circuits::bigint::{utils::{biguint_from_wires, random_u254, wires_set_from_u254}, U254};
    use super::*;

    #[test]
    fn test_equal_and_equal_constant() {
        let a = random_u254();
        let b = random_u254();
        let circuit = U254::equal(wires_set_from_u254(a.clone()), wires_set_from_u254(b.clone()));
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(a == b, circuit.0[0].borrow().get_value());

        let a = random_u254();
        let circuit = U254::equal(wires_set_from_u254(a.clone()), wires_set_from_u254(a.clone()));
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(true, circuit.0[0].borrow().get_value());

        let a = random_u254();
        let circuit = U254::equal_constant(wires_set_from_u254(a.clone()), b.clone());
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(a == b, circuit.0[0].borrow().get_value());
    }

    #[test]
    fn test_greater_than() {
        let a = random_u254();
        let b = random_u254();
        let circuit = U254::greater_than(wires_set_from_u254(a.clone()), wires_set_from_u254(b.clone()));
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(a > b, circuit.0[0].borrow().get_value());

        let a = random_u254();
        let circuit = U254::greater_than(wires_set_from_u254(a.clone()), wires_set_from_u254(a.clone()));
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(false, circuit.0[0].borrow().get_value());

        let a = random_u254();
        let circuit = U254::greater_than(wires_set_from_u254(a.clone() + BigUint::from_str("1").unwrap()), wires_set_from_u254(a.clone()));
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(true, circuit.0[0].borrow().get_value());
    }

    #[test]
    fn test_less_than_constant() {
        let a = random_u254();
        let b = random_u254();
        let circuit = U254::less_than_constant(wires_set_from_u254(a.clone()), b.clone());
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        assert_eq!(a < b, circuit.0[0].borrow().get_value());
    }

    #[test]
    fn test_select() {
        let a = random_u254();
        let b = random_u254();
        let s = Rc::new(RefCell::new(Wire::new()));
        s.borrow_mut().set(true);
        let circuit = U254::select(wires_set_from_u254(a.clone()), wires_set_from_u254(b.clone()), s);
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        let c = biguint_from_wires(circuit.0);
        assert_eq!(a, c);
    }

    #[test]
    fn test_self_or_zero() {
        let a = random_u254();

        let s = Rc::new(RefCell::new(Wire::new()));
        s.borrow_mut().set(true);
        let circuit = U254::self_or_zero(wires_set_from_u254(a.clone()), s);
        circuit.print_gate_type_counts();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        let c = biguint_from_wires(circuit.0);
        assert_eq!(a, c);

        let s = Rc::new(RefCell::new(Wire::new()));
        s.borrow_mut().set(false);
        let circuit = U254::self_or_zero(wires_set_from_u254(a.clone()), s);
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        let c = biguint_from_wires(circuit.0);
        assert_eq!(c, BigUint::ZERO);
    }
}
