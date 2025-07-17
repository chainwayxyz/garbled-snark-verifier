use crate::{
    bag::*,
    circuits::bigint::utils::{bits_from_biguint, n_wires},
};
use num_bigint::BigUint;
pub mod add;
pub mod cmp;
pub mod mul;
pub mod utils;

#[derive(Debug)]
pub struct BigIntImpl<const N_BITS: usize> {}

impl<const N_BITS: usize> BigIntImpl<N_BITS> {
    pub const N_BITS: usize = N_BITS;
    pub fn wires() -> Wires {
        n_wires(N_BITS)
    }
    pub fn wires_set_from_number(u: &BigUint) -> Wires {
        bits_from_biguint(u)[0..N_BITS]
            .iter()
            .map(|bit| {
                let wire = new_wirex();
                wire.borrow_mut().set(*bit);
                wire
            })
            .collect()
    }
}

pub type U254 = BigIntImpl<254>;
pub type U32 = BigIntImpl<32>;
