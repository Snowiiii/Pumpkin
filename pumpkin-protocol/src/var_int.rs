use std::io::{self, Write};

use bytes::Buf;
use thiserror::Error;

use crate::VarIntType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarInt(pub VarIntType);

impl VarInt {
    /// The maximum number of bytes a `VarInt` could occupy when read from and
    /// written to the Minecraft protocol.
    pub const MAX_SIZE: usize = 5;

    /// Returns the exact number of bytes this varint will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    pub const fn written_size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    pub fn decode_partial(r: &mut &[u8]) -> Result<i32, VarIntDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            if !r.has_remaining() {
                return Err(VarIntDecodeError::Incomplete);
            }
            let byte = r.get_u8();
            val |= (i32::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(val);
            }
        }

        Err(VarIntDecodeError::TooLarge)
    }

    pub fn encode(&self, mut w: impl Write) -> Result<(), io::Error> {
        let mut x = self.0 as u64;
        loop {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            if x == 0 {
                w.write_all(&[byte])?;
                break;
            }
            w.write_all(&[byte | 0x80])?;
        }
        Ok(())
    }

    pub fn decode(r: &mut &[u8]) -> Result<Self, VarIntDecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE {
            if !r.has_remaining() {
                return Err(VarIntDecodeError::Incomplete);
            }
            let byte = r.get_u8();
            val |= (i32::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(VarIntDecodeError::TooLarge)
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        VarInt(value as i32)
    }
}

impl From<u8> for VarInt {
    fn from(value: u8) -> Self {
        VarInt(value as i32)
    }
}

impl From<usize> for VarInt {
    fn from(value: usize) -> Self {
        VarInt(value as i32)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum VarIntDecodeError {
    #[error("incomplete VarInt decode")]
    Incomplete,
    #[error("VarInt is too large")]
    TooLarge,
}
