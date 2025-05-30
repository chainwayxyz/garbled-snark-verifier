use crate::{bag::*, circuits::{bigint::U254, bn254::fq::Fq}};

pub struct Fq2;

impl Fq2 {
    pub fn add(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*4);
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut a = input_wires[0..U254::N_BITS].to_vec();
        let mut b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        let c = input_wires[U254::N_BITS*2..U254::N_BITS*3].to_vec();
        let d = input_wires[U254::N_BITS*3..U254::N_BITS*4].to_vec();
        a.extend(c);
        b.extend(d);
        
        let (wires_1, gates_1) = Fq::add(a);
        circuit_gates.extend(gates_1);
        output_wires.extend(wires_1);
        let (wires_2 , gates_2) = Fq::add(b);
        circuit_gates.extend(gates_2);
        output_wires.extend(wires_2);
        (output_wires, circuit_gates)
    }

    pub fn sub(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*4);
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut a = input_wires[0..U254::N_BITS].to_vec();
        let mut b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        let c = input_wires[U254::N_BITS*2..U254::N_BITS*3].to_vec();
        let d = input_wires[U254::N_BITS*3..U254::N_BITS*4].to_vec();
        a.extend(c);
        b.extend(d);
        
        let (wires_1, gates_1) = Fq::sub(a);
        circuit_gates.extend(gates_1);
        output_wires.extend(wires_1);
        let (wires_2 , gates_2) = Fq::sub(b);
        circuit_gates.extend(gates_2);
        output_wires.extend(wires_2);
        (output_wires, circuit_gates)
    }

    pub fn double(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*2);
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let a = input_wires[0..U254::N_BITS].to_vec();
        let b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        
        let (wires_1, gates_1) = Fq::double(a);
        circuit_gates.extend(gates_1);
        output_wires.extend(wires_1);
        let (wires_2 , gates_2) = Fq::double(b);
        circuit_gates.extend(gates_2);
        output_wires.extend(wires_2);
        (output_wires, circuit_gates)
    }

    pub fn neg(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*2);
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let a = input_wires[0..U254::N_BITS].to_vec();
        let b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        
        let (wires_1, gates_1) = Fq::neg(a);
        circuit_gates.extend(gates_1);
        output_wires.extend(wires_1);
        let (wires_2 , gates_2) = Fq::neg(b);
        circuit_gates.extend(gates_2);
        output_wires.extend(wires_2);
        (output_wires, circuit_gates)
    }

    pub fn mul(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*4);
        let a = input_wires[0..U254::N_BITS].to_vec();
        let b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        let c = input_wires[U254::N_BITS*2..U254::N_BITS*3].to_vec();
        let d = input_wires[U254::N_BITS*3..U254::N_BITS*4].to_vec();
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut wire_ab = Vec::new();
        let mut wire_ac = Vec::new();
        let mut wire_bd = Vec::new();
        let mut wire_cd = Vec::new();
        wire_ab.extend(a.clone()); wire_ab.extend(b.clone());
        wire_ac.extend(a.clone()); wire_ac.extend(c.clone());
        wire_bd.extend(b.clone()); wire_bd.extend(d.clone());
        wire_cd.extend(c.clone()); wire_cd.extend(d.clone());

        let (mut wire_1, gate_1) = Fq::add(wire_ab);
        let (wire_2, gate_2) = Fq::add(wire_cd);
        let (mut wire_3, gate_3) = Fq::mul(wire_ac);
        let (wire_4, gate_4) = Fq::mul(wire_bd);
        wire_3.extend(wire_4.clone());
        let (wire_5, gate_5) = Fq::add(wire_3.clone());
        let (wire_6, gate_6) = Fq::sub(wire_3.clone());
        wire_1.extend(wire_2);
        let (mut wire_7, gate_7) = Fq::mul(wire_1);
        wire_7.extend(wire_5);
        let (wire_8, gate_8) = Fq::sub(wire_7);
        output_wires.extend(wire_6);
        output_wires.extend(wire_8);
        circuit_gates.extend(gate_1);
        circuit_gates.extend(gate_2);
        circuit_gates.extend(gate_3);
        circuit_gates.extend(gate_4);
        circuit_gates.extend(gate_5);
        circuit_gates.extend(gate_6);
        circuit_gates.extend(gate_7);
        circuit_gates.extend(gate_8);

        (output_wires, circuit_gates)
    }

    pub fn square(input_wires: Vec<Rc<RefCell<Wire>>>) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*2);
        let a = input_wires[0..U254::N_BITS].to_vec();
        let b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut wire_ab = Vec::new();
        let mut wire_ac = Vec::new();
        let mut wire_bd = Vec::new();
        let mut wire_cd = Vec::new();
        wire_ab.extend(a.clone()); wire_ab.extend(b.clone());
        wire_ac.extend(a.clone()); wire_ac.extend(a.clone());
        wire_bd.extend(b.clone()); wire_bd.extend(b.clone());
        wire_cd.extend(a.clone()); wire_cd.extend(b.clone());

        let (mut wire_1, gate_1) = Fq::add(wire_ab);
        let (wire_2, gate_2) = Fq::add(wire_cd);
        let (mut wire_3, gate_3) = Fq::mul(wire_ac);
        let (wire_4, gate_4) = Fq::mul(wire_bd);
        wire_3.extend(wire_4.clone());
        let (wire_5, gate_5) = Fq::add(wire_3.clone());
        let (wire_6, gate_6) = Fq::sub(wire_3.clone());
        wire_1.extend(wire_2);
        let (mut wire_7, gate_7) = Fq::mul(wire_1);
        wire_7.extend(wire_5);
        let (wire_8, gate_8) = Fq::sub(wire_7);
        output_wires.extend(wire_6);
        output_wires.extend(wire_8);
        circuit_gates.extend(gate_1);
        circuit_gates.extend(gate_2);
        circuit_gates.extend(gate_3);
        circuit_gates.extend(gate_4);
        circuit_gates.extend(gate_5);
        circuit_gates.extend(gate_6);
        circuit_gates.extend(gate_7);
        circuit_gates.extend(gate_8);

        (output_wires, circuit_gates)
    }

    pub fn mul_by_constant(input_wires: Vec<Rc<RefCell<Wire>>>, c: ark_bn254::Fq2 ) -> (Vec<Rc<RefCell<Wire>>>, Vec<Gate>) {
        assert_eq!( input_wires.len(), U254::N_BITS*2);
        let a = input_wires[0..U254::N_BITS].to_vec();
        let b = input_wires[U254::N_BITS..U254::N_BITS*2].to_vec();
        let x=c.c0;
        let y=c.c1;
        let mut circuit_gates = Vec::new();
        let mut output_wires = Vec::new();
        let mut wire_ab = Vec::new();
        wire_ab.extend(a.clone()); wire_ab.extend(b.clone());
        let (wire_1, gate_1) = Fq::add(wire_ab);
        let (mut wire_2, gate_2) = Fq::mul_by_constant(a,x);
        let (wire_3, gate_3) = Fq::mul_by_constant(b,y);
        let (mut wire_4, gate_4) = Fq::mul_by_constant(wire_1.clone(),x+y);
        wire_2.extend(wire_3.clone());
        let (wire_5, gate_5) = Fq::sub(wire_2.clone());
        let (wire_6, gate_6) = Fq::add(wire_2.clone());
        wire_4.extend(wire_6.clone());
        let (wire_7, gate_7) = Fq::sub(wire_4);


        output_wires.extend(wire_5);
        output_wires.extend(wire_7);
        circuit_gates.extend(gate_1);
        circuit_gates.extend(gate_2);
        circuit_gates.extend(gate_3);
        circuit_gates.extend(gate_4);
        circuit_gates.extend(gate_5);
        circuit_gates.extend(gate_6);
        circuit_gates.extend(gate_7);

        (output_wires, circuit_gates)
    }

}


#[cfg(test)]
mod tests {
    use crate::circuits::bn254::utils::{bits_from_fq2, fq2_from_bits, random_fq2};
    use super::*;

    #[test]
    fn test_fq2_add() {
        let a = random_fq2();
        let b = random_fq2();
        let c = a + b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        for bit in bits_from_fq2(b) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq2::add(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    }    

    #[test]
    fn test_fq2_sub() {
        let a = random_fq2();
        let b = random_fq2();
        let c = a - b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        for bit in bits_from_fq2(b) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq2::sub(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    }    

    #[test]
    fn test_fq2_double() {
        let a = random_fq2();
        let c = a + a;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
    
        let (output_wires, gates) = Fq2::double(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    }  

    #[test]
    fn test_fq2_neg() {
        let a = random_fq2();
        let c = -a;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
    
        let (output_wires, gates) = Fq2::neg(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    }

    #[test]
    fn test_fq2_mul() {
        let a = random_fq2();
        let b = random_fq2();
        let c = a * b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        for bit in bits_from_fq2(b) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq2::mul(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    } 

    #[test]
    fn test_fq2_square() {
        let a = random_fq2();
        let c = a * a;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq2::square(input_wires);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    } 

    #[test]
    fn test_fq2_mul_by_constant() {
        let a = random_fq2();
        let b = random_fq2();
        let c = a * b;
        let mut input_wires = Vec::new();
        for bit in bits_from_fq2(a) {
            let wire = Rc::new(RefCell::new(Wire::new()));
            wire.borrow_mut().set(bit);
            input_wires.push(wire)
        }
        let (output_wires, gates) = Fq2::mul_by_constant(input_wires, b);
        println!("gate count: {:?}", gates.len());
        for mut gate in gates {
            gate.evaluate();
        }
        let d = fq2_from_bits(output_wires.iter().map(|output_wire| { output_wire.borrow().get_value() }).collect());
        assert_eq!(c, d);
    } 
}

