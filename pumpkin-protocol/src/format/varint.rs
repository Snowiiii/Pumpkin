use std::{
    cmp,
    io::{self, Write},
};

use bytes::{Buf, BytesMut};
use serde::Serialize;
use thiserror::Error;

/**
 * A variable-length integer type used by the Minecraft network protocol.
 */

pub type VarIntType = i32;
pub const MAX_SIZE: usize = 5;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Error)]
pub enum DecodeError {
    #[error("incomplete VarInt decode")]
    Incomplete,
    #[error("VarInt is too large")]
    TooLarge,
}

/// A variable-length integer type used by the Minecraft network protocol.
/// This type is meant to be used like the following
///
/// ```rust
/// #[derive(FromPrimitive, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// #[serde(try_from = "VarInt")]
/// enum ConnectionState {
///    Handshaking = 0,
///    Status
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VarInt(VarIntType);

impl VarInt {
    /// Create a new VarInt from the provided value
    pub fn new(value: VarIntType) -> Self {
        VarInt(value)
    }

    pub const fn written_size(self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl TryFrom<&[u8]> for VarInt {
    type Error = DecodeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut val = 0;
        for i in 0..cmp::min(bytes.len(), MAX_SIZE) {
            let byte = bytes[i];
            val |= (VarIntType::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(VarInt(val));
            }
        }
        if bytes.len() >= MAX_SIZE {
            return Err(DecodeError::TooLarge);
        }
        Err(DecodeError::Incomplete)
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        varint::serialize(self.0, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        varint::deserialize(deserializer).map(VarInt)
    }
}

macro_rules! impl_conversions {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for VarInt {
                fn from(value: $ty) -> Self {
                    VarInt(value as i32)
                }
            }

            impl From<VarInt> for $ty {
                fn from(value: VarInt) -> Self {
                    value.0 as $ty
                }
            }
        )*
    };
    () => {};
}

impl_conversions!(i8, u8, i32, u32, i64, u64, usize, isize);

pub mod varint {
    use bytes::BufMut;
    use serde::de;
    use serde::Serializer;

    use super::{DecodeError, MAX_SIZE};

    pub fn serialize<S>(value: impl Into<i32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = [0; MAX_SIZE];
        let writer = buf.writer();
        super::encode(value.into(), writer).map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&buf)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VarIntVisitor;
        impl<'a> de::Visitor<'a> for VarIntVisitor {
            type Value = i32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'a>,
            {
                let mut buffer = [0; super::MAX_SIZE];
                for idx in 0..super::MAX_SIZE {
                    buffer[idx] = seq
                        .next_element::<u8>()?
                        .ok_or(de::Error::custom("Parsing VarInt failed"))?;
                    match super::decode(&buffer[..idx + 1]) {
                        // TODO: Run through till byte & 0b1000_0000 == 0
                        Ok(val) => return Ok(val),
                        Err(DecodeError::Incomplete) => continue,
                        Err(DecodeError::TooLarge) => {
                            return Err(de::Error::custom("VarInt is too large"))
                        }
                    }
                }
                Err(de::Error::custom("VarInt is too large"))
            }
        }
        deserializer.deserialize_seq(VarIntVisitor)
    }
}

/// Encode a value as a variable-length integer into the provided buffer
pub fn encode(value: impl Into<i32>, mut w: impl Write) -> Result<(), io::Error> {
    let mut buffer = [0; MAX_SIZE];
    let mut value = value.into();
    for idx in 0..MAX_SIZE {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            w.write_all(&buffer[..idx + 1])?;
            break;
        }
        buffer[idx] = byte | 0x80;
    }
    Ok(())
}

/// Decode a variable-length integer from the provided buffer
pub fn decode(mut r: impl Buf) -> Result<VarIntType, DecodeError> {
    let mut val = 0;
    for i in 0..MAX_SIZE {
        if !r.has_remaining() {
            return Err(DecodeError::Incomplete);
        }
        let byte = r.get_u8();
        val |= (i32::from(byte) & 0b01111111) << (i * 7);
        if byte & 0b10000000 == 0 {
            return Ok(val);
        }
    }
    Err(DecodeError::TooLarge)
}

/// Try to decode a variable-length integer from the byte buffer
pub fn try_decode(buffer: &mut BytesMut) -> Result<VarIntType, DecodeError> {
    let mut val = 0;
    for i in 0..cmp::min(buffer.len(), MAX_SIZE) {
        let byte = buffer[i];
        val |= (VarIntType::from(byte) & 0b01111111) << (i * 7);
        if byte & 0b1000_0000 == 0 {
            buffer.advance(i + 1);
            return Ok(val);
        }
    }
    if buffer.len() >= MAX_SIZE {
        return Err(DecodeError::TooLarge);
    }
    Err(DecodeError::Incomplete)
}
