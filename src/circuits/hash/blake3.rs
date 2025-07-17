use crate::{bag::*, circuits::{bigint::U32, hash::blake3x::{IV, MSG_PERMUTATION}}};

pub fn wires_set_u32(a: u32) -> Wires {
    let mut bits = Vec::new();
    for byte in a.to_le_bytes() {
        for i in 0..8 {
            bits.push(((byte >> i) & 1) == 1)
        }
    }
    bits.iter().map(|bit| {
        let wire = new_wirex();
        wire.borrow_mut().set(*bit);
        wire
    }).collect()
}

pub fn wires_set_u8(a: u8) -> Wires {
    let mut wires = Vec::new();
    for i in 0..8 {
        let wire = new_wirex();
        wire.borrow_mut().set(((a >> i) & 1) == 1);
        wires.push(wire);
    }
    wires
}

pub fn u32_from_wires(a: Wires) -> u32 {
    let mut u = 0;
    for wire in a.iter().rev() {
        let bit = wire.borrow().get_value();
        u = u + u + if bit { 1 } else { 0 };
    }
    u
}

pub fn u8_from_wires(a: Wires) -> u8 {
    let mut u = 0;
    for wire in a.iter().rev() {
        let bit = wire.borrow().get_value();
        u = u + u + if bit { 1 } else { 0 };
    }
    u
}

pub fn u32_rotate_right(a: Wires, i: usize) -> Circuit {
    assert_eq!(a.len(), 32);
    let mut circuit =Circuit::empty();
    let u = a[0..i].to_vec();
    let mut v = a[i..].to_vec();
    v.extend(u);
    circuit.add_wires(v);
    circuit
}

pub fn u32_xor(a: Wires, b: Wires) -> Circuit {
    assert_eq!(a.len(), 32);
    assert_eq!(b.len(), 32);
    let mut circuit =Circuit::empty();
    let mut c = Vec::new();
    for i in 0..32 {
        let ci = new_wirex();
        circuit.add(Gate::xor(a[i].clone(), b[i].clone(), ci.clone()));
        c.push(ci);
    }
    circuit.add_wires(c);
    circuit
}

pub fn g_circuit(state: &mut [Wires; 16], a: usize, b: usize, c: usize, d: usize, x: Wires, y: Wires) -> Circuit {
    assert_eq!(x.len(), 32);
    assert_eq!(y.len(), 32);
    let mut circuit = Circuit::empty();

    let (ai, bi, ci, di) = (a, b, c, d);

    let (a, b, c, d) = (state[a].clone(), state[b].clone(), state[c].clone(), state[d].clone());

    let a_plus_b = circuit.extend(U32::add_without_carry(a.clone(), b.clone()));
    let a = circuit.extend(U32::add_without_carry(a_plus_b, x));
    let d_xor_a = circuit.extend(u32_xor(d.clone(), a.clone()));
    let d = circuit.extend(u32_rotate_right(d_xor_a, 16));
    let c = circuit.extend(U32::add_without_carry(c.clone(), d.clone()));
    let b_xor_c = circuit.extend(u32_xor(b.clone(), c.clone()));
    let b = circuit.extend(u32_rotate_right(b_xor_c, 12));

    let a_plus_b = circuit.extend(U32::add_without_carry(a.clone(), b.clone()));
    let a = circuit.extend(U32::add_without_carry(a_plus_b, y));
    let d_xor_a = circuit.extend(u32_xor(d.clone(), a.clone()));
    let d = circuit.extend(u32_rotate_right(d_xor_a, 8));
    let c = circuit.extend(U32::add_without_carry(c.clone(), d.clone()));
    let b_xor_c = circuit.extend(u32_xor(b.clone(), c.clone()));
    let b = circuit.extend(u32_rotate_right(b_xor_c, 7));

    state[ai] = a;
    state[bi] = b;
    state[ci] = c;
    state[di] = d;
 
    circuit
}

pub fn round_circuit(state: &mut [Wires; 16], m: [Wires; 16]) -> Circuit {
    let mut circuit = Circuit::empty();
    circuit.extend(g_circuit(state, 0, 4, 8, 12, m[0].clone(), m[1].clone()));
    circuit.extend(g_circuit(state, 1, 5, 9, 13, m[2].clone(), m[3].clone()));
    circuit.extend(g_circuit(state, 2, 6, 10, 14, m[4].clone(), m[5].clone()));
    circuit.extend(g_circuit(state, 3, 7, 11, 15, m[6].clone(), m[7].clone()));
    circuit.extend(g_circuit(state, 0, 5, 10, 15, m[8].clone(), m[9].clone()));
    circuit.extend(g_circuit(state, 1, 6, 11, 12, m[10].clone(), m[11].clone()));
    circuit.extend(g_circuit(state, 2, 7, 8, 13, m[12].clone(), m[13].clone()));
    circuit.extend(g_circuit(state, 3, 4, 9, 14, m[14].clone(), m[15].clone()));
    circuit
}

pub fn permute(m: &mut [Wires; 16]) {
    let mut permuted = [const {vec![]}; 16];
    for i in 0..16 {
        permuted[i] = m[MSG_PERMUTATION[i]].clone();
    }
    *m = permuted;
}

pub fn compress_circuit(
    h: [Wires; 8],
    block: [Wires; 16],
) -> Circuit {
    let mut circuit = Circuit::empty();
    let mut state = [
        h[0].clone(),         h[1].clone(),         h[2].clone(),         h[3].clone(),
        h[4].clone(),         h[5].clone(),         h[6].clone(),         h[7].clone(),
        wires_set_u32(IV[0]), wires_set_u32(IV[1]), wires_set_u32(IV[2]), wires_set_u32(IV[3]),
        wires_set_u32(0),     wires_set_u32(0),     wires_set_u32(64),    wires_set_u32(11),
    ];
    let mut block = block;

    circuit.extend(round_circuit(&mut state, block.clone()));
    permute(&mut block);
    circuit.extend(round_circuit(&mut state, block.clone()));
    permute(&mut block);
    circuit.extend(round_circuit(&mut state, block.clone()));
    permute(&mut block);
    circuit.extend(round_circuit(&mut state, block.clone()));
    permute(&mut block);
    circuit.extend(round_circuit(&mut state, block.clone()));
    permute(&mut block);
    circuit.extend(round_circuit(&mut state, block.clone()));
    permute(&mut block);
    circuit.extend(round_circuit(&mut state, block.clone()));

    for i in 0..8 {
        state[i] = circuit.extend(u32_xor(state[i].clone(), state[i + 8].clone()));
        state[i + 8] = circuit.extend(u32_xor(state[i + 8].clone(), h[i].clone()));
    }

    for s in state {
        circuit.add_wires(s);
    }
    circuit
}

pub fn blake3_512_circuit(input: Wires) -> Circuit {
    assert_eq!(input.len(), 512);
    let block = input.chunks(32).map(|s| { s.to_vec() }).collect::<Vec<_>>().try_into().unwrap();
    let mut circuit = Circuit::empty();
    let iv_wires = IV.iter().map(|u| wires_set_u32(*u)).collect::<Vec<_>>().try_into().unwrap();
    let words = circuit.extend(compress_circuit(iv_wires, block));
    circuit.add_wires(words[0..256].to_vec());
    circuit
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rand::{rng, Rng};
    use crate::circuits::hash::{blake3::{blake3_512_circuit, compress_circuit, g_circuit, round_circuit, u32_from_wires, u8_from_wires, wires_set_u32, wires_set_u8}, blake3x::{blake3_512, compress, round}};
    use crate::circuits::hash::blake3x::g;

    #[test]
    fn test_g_circuit() {
        let a = rng().random();
        let b = rng().random();
        let c = rng().random();
        let d = rng().random();
        let x = rng().random();
        let y = rng().random();
        let a_wires = wires_set_u32(a);
        let b_wires = wires_set_u32(b);
        let c_wires = wires_set_u32(c);
        let d_wires = wires_set_u32(d);
        let x_wires = wires_set_u32(x);
        let y_wires = wires_set_u32(y);
        let mut state = [a, b, c, d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut state_wires = [a_wires, b_wires, c_wires, d_wires, vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
        let circuit = g_circuit(&mut state_wires, 0, 1, 2, 3, x_wires, y_wires);
        circuit.gate_counts().print();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        let new_a = u32_from_wires(state_wires[0].clone());
        let new_b = u32_from_wires(state_wires[1].clone());
        let new_c = u32_from_wires(state_wires[2].clone());
        let new_d = u32_from_wires(state_wires[3].clone());
        let new_state = [new_a, new_b, new_c, new_d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        g(&mut state, 0, 1, 2, 3, x, y);
        assert_eq!(state, new_state);
    }

    #[test]
    fn test_round_circuit() {
        let mut state = [rng().random(); 16];
        let m = [rng().random(); 16];
        let mut state_wires = [const {vec![]}; 16];
        let mut m_wires = [const {vec![]}; 16];
        for i in 0..16 {
            state_wires[i] = wires_set_u32(state[i]);
            m_wires[i] = wires_set_u32(m[i]);
        }
        let circuit = round_circuit(&mut state_wires, m_wires);
        circuit.gate_counts().print();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        round(&mut state, &m);
        let state2: Vec<u32> = state_wires.iter().map(|s| { u32_from_wires(s.clone()) }).collect();
        assert_eq!(state.to_vec(), state2);
    }

    #[test]
    fn test_compress_circuit() {
        let h = [rng().random(); 8];
        let block = [rng().random(); 16];
        let mut h_wires = [const {vec![]}; 8];
        let mut block_wires = [const {vec![]}; 16];
        for i in 0..8 {
            h_wires[i] = wires_set_u32(h[i]);
        }
        for i in 0..16 {
            block_wires[i] = wires_set_u32(block[i]);
        }
        let circuit = compress_circuit(h_wires, block_wires);
        circuit.gate_counts().print();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        let mut a = [0; 16];
        for i in 0..16 {
            a[i] = u32_from_wires(circuit.0[32*i..32*(i+1)].to_vec());
        }
        let b = compress(&h, &block, 0, 64, 11);
        assert_eq!(a, b);
    }

    #[test]
    fn test_blake3_circuit() {
        let input = [rng().random::<u8>(); 64];
        let input_wires = input.iter().map(|u| { wires_set_u8(*u) }).concat();
        let circuit = blake3_512_circuit(input_wires);
        circuit.gate_counts().print();
        for mut gate in circuit.1 {
            gate.evaluate();
        }
        let output1 = circuit.0.chunks(8).map(|chunk| { u8_from_wires(chunk.to_vec()) }).collect::<Vec<u8>>();
        let output2 = blake3_512(input);
        assert_eq!(output1, output2.to_vec());
    }
}

