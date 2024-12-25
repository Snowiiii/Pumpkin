use std::num::NonZeroUsize;

use bytes::{Buf, BufMut};
use serde::{Serialize, Serializer};

use crate::bytebuf::ByteBuf;
use crate::bytebuf::ByteBufMut;

use super::{var_int::VarInt, Codec, DecodeError};

pub struct BitSet(pub VarInt, pub Vec<i64>);

impl Codec<BitSet> for BitSet {
    /// The maximum size of the BitSet is `remaining / 8`.
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(usize::MAX) };

    fn written_size(&self) -> usize {
        todo!()
    }

    fn encode(&self, write: &mut impl BufMut) {
        write.put_var_int(&self.0);
        for b in &self.1 {
            write.put_i64(*b);
        }
    }

    fn decode(read: &mut impl Buf) -> Result<Self, DecodeError> {
        // read length
        let length = read
            .try_get_var_int()
            .map_err(|_| DecodeError::Incomplete)?;
        // vanilla uses remaining / 8
        if length.0 as usize >= read.remaining() / 8 {
            return Err(DecodeError::TooLarge);
        }
        let mut array: Vec<i64> = Vec::with_capacity(size_of::<i64>() * length.0 as usize);
        for _ in 0..length.0 {
            let long = read.try_get_i64().map_err(|_| DecodeError::Incomplete)?;
            array.push(long);
        }
        Ok(BitSet(length, array))
    }
}

impl Serialize for BitSet {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}
