use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token};

#[derive(serde::Deserialize)]
struct WireId(usize);

#[derive(serde::Deserialize)]
struct Gate {
    wire_a: WireId,
    wire_b: WireId,
    wire_c: WireId,
    gate_type: String,
}

#[derive(serde::Deserialize)]
struct StoredGates {
    gates: Vec<Gate>,
    input_wire_ids: Vec<WireId>,
}

pub struct CommitmentMacroInput {
    path: LitStr,
    gate_hasher: Expr,
    delta: Expr,
    false_wire: Expr,
    true_wire: Expr,
    input_wires: Expr,
}

impl Parse for CommitmentMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let gate_hasher: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let delta: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let false_wire: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let true_wire: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let input_wires: Expr = input.parse()?;
        let _ = input.parse::<Token![,]>(); // optional trailing comma

        Ok(CommitmentMacroInput {
            path,
            gate_hasher,
            delta,
            false_wire,
            true_wire,
            input_wires,
        })
    }
}

pub fn generate_commitment_impl(input: CommitmentMacroInput) -> syn::Result<TokenStream> {
    let path_val = input.path.value();
    let file_path = if path_val.starts_with("OUT_DIR/") {
        let out_dir = std::env::var("OUT_DIR")
            .map_err(|_| syn::Error::new_spanned(&input.path, "OUT_DIR not found"))?;
        std::path::Path::new(&out_dir).join(&path_val[8..])
    } else {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        std::path::Path::new(&manifest_dir).join(&path_val)
    };

    let file_content = std::fs::read_to_string(&file_path).map_err(|e| {
        syn::Error::new(
            input.path.span(),
            format!(
                "Failed to read circuit gates file at {}: {}",
                file_path.display(),
                e
            ),
        )
    })?;

    let stored_gates: StoredGates = serde_json::from_str(&file_content).map_err(|e| {
        syn::Error::new(
            input.path.span(),
            format!("Failed to parse JSON from {}: {}", file_path.display(), e),
        )
    })?;

    let gate_hasher = input.gate_hasher;
    let delta = input.delta;
    let false_wire = input.false_wire;
    let true_wire = input.true_wire;
    let input_wires = input.input_wires;

    // Track the highest wire ID just to size any vectors if needed, though we will just emit local variables.
    let mut declarations = Vec::new();

    // Map input wires
    for (i, wire_id) in stored_gates.input_wire_ids.iter().enumerate() {
        let ident = format_ident!("w_{}", wire_id.0);
        declarations.push(quote! {
            let mut #ident = #input_wires[#i];
        });
    }

    let mut garble_statements = Vec::new();
    let mut last_generated_wire = format_ident!("w_0"); // Default

    let false_ident = format_ident!("w_{}", 0usize); // 0 is FALSE_WIRE
    let true_ident = format_ident!("w_{}", 1usize); // 1 is TRUE_WIRE

    // FALSE_WIRE and TRUE_WIRE constants are 0 and 1 in the main crate (`crate::circuit::FALSE_WIRE.0`).
    // In our JSON, wire IDs will be exactly what was recorded. So we assume WireId(0) is false, WireId(1) is true.
    declarations.push(quote! {
        let mut #false_ident = #false_wire;
        let mut #true_ident = #true_wire;
    });

    for (gate_index, gate) in stored_gates.gates.iter().enumerate() {
        // UNREACHABLE wire_id is usize::MAX
        if gate.wire_c.0 == usize::MAX {
            continue;
        }

        let a_ident = format_ident!("w_{}", gate.wire_a.0);
        let b_ident = format_ident!("w_{}", gate.wire_b.0);
        let c_ident = format_ident!("w_{}", gate.wire_c.0);

        // Ensure wire_c is declared
        // To be completely safe and allow reassignment or new assignment:
        // We can just use `let #c_ident = ...` and allow shadowing if it was declared.
        // Shadowing is perfectly fine in Rust and prevents "undeclared variable" errors if a wire ID was not yet seen.

        let gate_type_ident = format_ident!("{}", gate.gate_type);

        garble_statements.push(quote! {
            let (#c_ident, ct_opt) = garble_gate(
                #gate_hasher,
                GateType::#gate_type_ident,
                #a_ident,
                #b_ident,
                #delta,
                #gate_index
            );

            if let Some(ct) = ct_opt {
                ciphertexts.push(ct);
            }
        });

        last_generated_wire = c_ident;
    }

    let gate_count = stored_gates.gates.len();

    // Wrap everything in a block to isolate variables
    let expanded = quote! {
        {
            #( #declarations )*

            let mut ciphertexts = Vec::with_capacity(#gate_count); // Overestimate, that's fine

            #( #garble_statements )*

            (ciphertexts, #last_generated_wire)
        }
    };
    Ok(expanded)
}
