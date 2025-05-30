pub mod add;
pub mod cmp;
pub mod mul;
pub mod sub;
pub mod utils;

#[derive(Debug)]
pub struct BigIntImpl<const N_BITS: usize> {}

impl<const N_BITS: usize> BigIntImpl<N_BITS> {
    pub const N_BITS: usize = N_BITS;
}

pub type U254 = BigIntImpl<254>;
