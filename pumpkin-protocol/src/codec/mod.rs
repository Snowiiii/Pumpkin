use std::num::NonZeroUsize;

use bytes::{Buf, BufMut};
use thiserror::Error;

pub mod bit_set;
pub mod identifier;
pub mod slot;
pub mod var_int;
pub mod var_long;

pub trait Codec<T> {
    const MAX_SIZE: NonZeroUsize;

    fn written_size(&self) -> usize;

    fn encode(&self, write: &mut impl BufMut);

    fn decode(read: &mut impl Buf) -> Result<T, DecodeError>;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum DecodeError {
    #[error("Incomplete VarInt decode")]
    Incomplete,
    #[error("VarInt is too large")]
    TooLarge,
}
