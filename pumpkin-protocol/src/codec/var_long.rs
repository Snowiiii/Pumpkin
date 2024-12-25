use std::{num::NonZeroUsize, ops::Deref};

use super::{Codec, DecodeError};
use bytes::{Buf, BufMut};
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

pub type VarLongType = i64;

/**
 * A variable-length long type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarLong(pub VarLongType);

impl Codec<Self> for VarLong {
    /// The maximum number of bytes a `VarLong` can occupy.
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };

    /// Returns the exact number of bytes this varlong will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    fn encode(&self, write: &mut impl BufMut) {
        let mut x = self.0;
        for _ in 0..Self::MAX_SIZE.get() {
            let byte = (x & 0x7F) as u8;
            x >>= 7;
            if x == 0 {
                write.put_slice(&[byte]);
                break;
            }
            write.put_slice(&[byte | 0x80]);
        }
    }

    fn decode(read: &mut impl Buf) -> Result<Self, DecodeError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            if !read.has_remaining() {
                return Err(DecodeError::Incomplete);
            }
            let byte = read.get_u8();
            val |= (i64::from(byte) & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(VarLong(val));
            }
        }
        Err(DecodeError::TooLarge)
    }
}

impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        VarLong(value)
    }
}

impl From<u32> for VarLong {
    fn from(value: u32) -> Self {
        VarLong(value as i64)
    }
}

impl From<u8> for VarLong {
    fn from(value: u8) -> Self {
        VarLong(value as i64)
    }
}

impl From<usize> for VarLong {
    fn from(value: usize) -> Self {
        VarLong(value as i64)
    }
}

impl From<VarLong> for i64 {
    fn from(value: VarLong) -> Self {
        value.0
    }
}

impl AsRef<i64> for VarLong {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Deref for VarLong {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for VarLong {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut value = self.0 as u64;
        let mut buf = Vec::new();

        while value > 0x7F {
            buf.put_u8(value as u8 | 0x80);
            value >>= 7;
        }

        buf.put_u8(value as u8);

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for VarLong {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarLongVisitor;

        impl<'de> Visitor<'de> for VarLongVisitor {
            type Value = VarLong;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut val = 0;
                for i in 0..VarLong::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        val |= (i64::from(byte) & 0b01111111) << (i * 7);
                        if byte & 0b10000000 == 0 {
                            return Ok(VarLong(val));
                        }
                    } else {
                        break;
                    }
                }
                Err(de::Error::custom("VarInt was too large"))
            }
        }

        deserializer.deserialize_seq(VarLongVisitor)
    }
}
