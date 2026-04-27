use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    // 1. Write test_gates.json
    let test_gates = r#"{
      "gates": [
        { "wire_a": 2, "wire_b": 3, "wire_c": 5, "gate_type": "And" },
        { "wire_a": 5, "wire_b": 4, "wire_c": 6, "gate_type": "Xor" }
      ],
      "input_wire_ids": [2, 3, 4]
    }"#;
    let dest_path = PathBuf::from(&out_dir).join("test_gates.json");
    fs::write(&dest_path, test_gates).unwrap();

    // 2. Generate 64-bit adder circuit using helper functions mirroring src/gadgets/basic.rs
    let num_bits = 64;
    let mut gates = Vec::new();
    let mut next_wire = 2 + num_bits * 2;

    let a_wires: Vec<usize> = (2..2 + num_bits).collect();
    let b_wires: Vec<usize> = (2 + num_bits..2 + num_bits * 2).collect();

    // Replicate BigInt::add logic
    let (_res0, mut carry) = half_adder(&mut gates, &mut next_wire, a_wires[0], b_wires[0]);

    for i in 1..num_bits {
        let (_res, new_carry) =
            full_adder(&mut gates, &mut next_wire, a_wires[i], b_wires[i], carry);
        carry = new_carry;
    }

    let input_wire_ids: Vec<String> = (2..2 + num_bits * 2).map(|i| i.to_string()).collect();
    let adder_json = format!(
        "{{\n  \"gates\": [\n{}\n  ],\n  \"input_wire_ids\": [{}]\n}}",
        gates.join(",\n"),
        input_wire_ids.join(", ")
    );

    let adder_path = PathBuf::from(&out_dir).join("adder_64.json");
    fs::write(&adder_path, adder_json).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

// Helpers mirroring src/gadgets/basic.rs
fn half_adder(
    gates: &mut Vec<String>,
    next_wire: &mut usize,
    a: usize,
    b: usize,
) -> (usize, usize) {
    let res = *next_wire;
    *next_wire += 1;
    let carry = *next_wire;
    *next_wire += 1;
    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"Xor\" }}",
        a, b, res
    ));
    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"And\" }}",
        a, b, carry
    ));
    (res, carry)
}

fn full_adder(
    gates: &mut Vec<String>,
    next_wire: &mut usize,
    a: usize,
    b: usize,
    c: usize,
) -> (usize, usize) {
    // Exactly mirroring full_adder in basic.rs: [axc, bxc, result, t, carry]
    let axc = *next_wire;
    *next_wire += 1;
    let bxc = *next_wire;
    *next_wire += 1;
    let res = *next_wire;
    *next_wire += 1;
    let t = *next_wire;
    *next_wire += 1;
    let carry = *next_wire;
    *next_wire += 1;

    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"Xor\" }}",
        a, c, axc
    ));
    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"Xor\" }}",
        b, c, bxc
    ));
    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"Xor\" }}",
        a, bxc, res
    ));
    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"And\" }}",
        axc, bxc, t
    ));
    gates.push(format!(
        "    {{ \"wire_a\": {}, \"wire_b\": {}, \"wire_c\": {}, \"gate_type\": \"Xor\" }}",
        c, t, carry
    ));

    (res, carry)
}
