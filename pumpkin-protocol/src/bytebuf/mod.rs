use core::str;

use crate::{BitSet, FixedBitSet, VarInt, VarLong};
use bytes::{Buf, BufMut};

mod deserializer;
use thiserror::Error;
pub mod packet_id;
mod serializer;

use std::mem::size_of;

#[derive(Debug, Error)]
pub enum ReadingError {
    /// End-of-File
    #[error("EOF, Tried to read {0} but No bytes left to consume")]
    EOF(String),
    #[error("{0} is Incomplete")]
    Incomplete(String),
    #[error("{0} is too Large")]
    TooLarge(String),
    #[error("{0}")]
    Message(String),
}

pub trait ByteBuf: Buf {
    fn try_get_bool(&mut self) -> Result<bool, ReadingError>;

    fn try_get_u8(&mut self) -> Result<u8, ReadingError>;

    fn try_get_i8(&mut self) -> Result<i8, ReadingError>;

    fn try_get_u16(&mut self) -> Result<u16, ReadingError>;

    fn try_get_i16(&mut self) -> Result<i16, ReadingError>;

    fn try_get_u32(&mut self) -> Result<u32, ReadingError>;

    fn try_get_i32(&mut self) -> Result<i32, ReadingError>;

    fn try_get_u64(&mut self) -> Result<u64, ReadingError>;

    fn try_get_i64(&mut self) -> Result<i64, ReadingError>;

    fn try_get_f32(&mut self) -> Result<f32, ReadingError>;

    fn try_get_f64(&mut self) -> Result<f64, ReadingError>;

    fn try_copy_to_bytes(&mut self, len: usize) -> Result<bytes::Bytes, ReadingError>;

    fn try_copy_to_slice(&mut self, dst: &mut [u8]) -> Result<(), ReadingError>;

    fn try_get_var_int(&mut self) -> Result<VarInt, ReadingError>;

    fn try_get_var_long(&mut self) -> Result<VarLong, ReadingError>;

    fn try_get_string(&mut self) -> Result<String, ReadingError>;

    fn try_get_string_len(&mut self, max_size: u32) -> Result<String, ReadingError>;

    /// Reads a boolean. If true, the closure is called, and the returned value is
    /// wrapped in Some. Otherwise, this returns None.
    fn try_get_option<G>(
        &mut self,
        val: impl FnOnce(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Option<G>, ReadingError>;

    fn get_list<G>(
        &mut self,
        val: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError>;

    fn try_get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError>;

    fn try_get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError>;
}

impl<T: Buf> ByteBuf for T {
    fn try_get_bool(&mut self) -> Result<bool, ReadingError> {
        Ok(self.try_get_u8()? != 0)
    }

    fn try_get_u8(&mut self) -> Result<u8, ReadingError> {
        if size_of::<u8>() <= self.remaining() {
            Ok(self.get_u8())
        } else {
            Err(ReadingError::EOF("u8".to_string()))
        }
    }

    fn try_get_i8(&mut self) -> Result<i8, ReadingError> {
        if size_of::<i8>() <= self.remaining() {
            Ok(self.get_i8())
        } else {
            Err(ReadingError::EOF("i8".to_string()))
        }
    }

    fn try_get_u16(&mut self) -> Result<u16, ReadingError> {
        if size_of::<u16>() <= self.remaining() {
            Ok(self.get_u16())
        } else {
            Err(ReadingError::EOF("u16".to_string()))
        }
    }

    fn try_get_i16(&mut self) -> Result<i16, ReadingError> {
        if size_of::<i16>() <= self.remaining() {
            Ok(self.get_i16())
        } else {
            Err(ReadingError::EOF("i16".to_string()))
        }
    }

    fn try_get_u32(&mut self) -> Result<u32, ReadingError> {
        if size_of::<u32>() <= self.remaining() {
            Ok(self.get_u32())
        } else {
            Err(ReadingError::EOF("u32".to_string()))
        }
    }

    fn try_get_i32(&mut self) -> Result<i32, ReadingError> {
        if size_of::<i32>() <= self.remaining() {
            Ok(self.get_i32())
        } else {
            Err(ReadingError::EOF("i32".to_string()))
        }
    }

    fn try_get_u64(&mut self) -> Result<u64, ReadingError> {
        if size_of::<u64>() <= self.remaining() {
            Ok(self.get_u64())
        } else {
            Err(ReadingError::EOF("u64".to_string()))
        }
    }

    fn try_get_i64(&mut self) -> Result<i64, ReadingError> {
        if size_of::<i64>() <= self.remaining() {
            Ok(self.get_i64())
        } else {
            Err(ReadingError::EOF("i64".to_string()))
        }
    }

    fn try_get_f32(&mut self) -> Result<f32, ReadingError> {
        if size_of::<f32>() <= self.remaining() {
            Ok(self.get_f32())
        } else {
            Err(ReadingError::EOF("f32".to_string()))
        }
    }

    fn try_get_f64(&mut self) -> Result<f64, ReadingError> {
        if size_of::<f64>() <= self.remaining() {
            Ok(self.get_f64())
        } else {
            Err(ReadingError::EOF("f64".to_string()))
        }
    }

    fn try_copy_to_bytes(&mut self, len: usize) -> Result<bytes::Bytes, ReadingError> {
        if self.remaining() >= len {
            Ok(self.copy_to_bytes(len))
        } else {
            Err(ReadingError::Message("Unable to copy bytes".to_string()))
        }
    }

    fn try_copy_to_slice(&mut self, dst: &mut [u8]) -> Result<(), ReadingError> {
        if self.remaining() >= dst.len() {
            self.copy_to_slice(dst);
            Ok(())
        } else {
            Err(ReadingError::Message("Unable to copy slice".to_string()))
        }
    }

    fn try_get_var_int(&mut self) -> Result<VarInt, ReadingError> {
        match VarInt::decode(self) {
            Ok(var_int) => Ok(var_int),
            Err(error) => match error {
                crate::VarIntDecodeError::Incomplete => {
                    Err(ReadingError::Incomplete("varint".to_string()))
                }
                crate::VarIntDecodeError::TooLarge => {
                    Err(ReadingError::TooLarge("varint".to_string()))
                }
            },
        }
    }
    fn try_get_var_long(&mut self) -> Result<VarLong, ReadingError> {
        match VarLong::decode(self) {
            Ok(var_long) => Ok(var_long),
            Err(error) => match error {
                crate::VarLongDecodeError::Incomplete => {
                    Err(ReadingError::Incomplete("varint".to_string()))
                }
                crate::VarLongDecodeError::TooLarge => {
                    Err(ReadingError::TooLarge("varlong".to_string()))
                }
            },
        }
    }

    fn try_get_string(&mut self) -> Result<String, ReadingError> {
        self.try_get_string_len(i16::MAX as u32)
    }

    fn try_get_string_len(&mut self, max_size: u32) -> Result<String, ReadingError> {
        let size = self.try_get_var_int()?.0;
        if size as u32 > max_size {
            return Err(ReadingError::TooLarge("string".to_string()));
        }

        let data = self.try_copy_to_bytes(size as usize)?;
        if data.len() as u32 > max_size {
            return Err(ReadingError::TooLarge("string".to_string()));
        }
        match str::from_utf8(&data) {
            Ok(string_result) => Ok(string_result.to_string()),
            Err(e) => Err(ReadingError::Message(e.to_string())),
        }
    }

    fn try_get_option<G>(
        &mut self,
        val: impl FnOnce(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Option<G>, ReadingError> {
        if self.try_get_bool()? {
            Ok(Some(val(self)?))
        } else {
            Ok(None)
        }
    }

    fn get_list<G>(
        &mut self,
        val: impl Fn(&mut Self) -> Result<G, ReadingError>,
    ) -> Result<Vec<G>, ReadingError> {
        let len = self.try_get_var_int()?.0 as usize;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(val(self)?);
        }
        Ok(list)
    }

    fn try_get_uuid(&mut self) -> Result<uuid::Uuid, ReadingError> {
        let mut bytes = [0u8; 16];
        self.try_copy_to_slice(&mut bytes)?;
        Ok(uuid::Uuid::from_slice(&bytes).expect("Failed to parse UUID"))
    }

    fn try_get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, ReadingError> {
        self.try_copy_to_bytes(bits.div_ceil(8))
    }
}

pub trait ByteBufMut {
    fn put_bool(&mut self, v: bool);

    fn put_uuid(&mut self, v: &uuid::Uuid);

    fn put_string(&mut self, val: &str);

    fn put_string_len(&mut self, val: &str, max_size: u32);

    fn put_string_array(&mut self, array: &[String]);

    fn put_bit_set(&mut self, set: &BitSet);

    /// Writes `true` if the option is Some, or `false` if None. If the option is
    /// some, then it also calls the `write` closure.
    fn put_option<G>(&mut self, val: &Option<G>, write: impl FnOnce(&mut Self, &G));

    fn put_list<G>(&mut self, list: &[G], write: impl Fn(&mut Self, &G));

    fn put_var_int(&mut self, value: &VarInt);

    fn put_varint_arr(&mut self, v: &[i32]);
}

impl<T: BufMut> ByteBufMut for T {
    fn put_bool(&mut self, v: bool) {
        if v {
            self.put_u8(1);
        } else {
            self.put_u8(0);
        }
    }

    fn put_uuid(&mut self, v: &uuid::Uuid) {
        // thats the vanilla way
        let pair = v.as_u64_pair();
        self.put_u64(pair.0);
        self.put_u64(pair.1);
    }

    fn put_string(&mut self, val: &str) {
        self.put_string_len(val, i16::MAX as u32);
    }

    fn put_string_len(&mut self, val: &str, max_size: u32) {
        if val.len() as u32 > max_size {
            // Should be panic?, I mean its our fault
            panic!("String is too big");
        }
        self.put_var_int(&val.len().into());
        self.put(val.as_bytes());
    }

    fn put_string_array(&mut self, array: &[String]) {
        for string in array {
            self.put_string(string)
        }
    }

    fn put_var_int(&mut self, value: &VarInt) {
        value.encode(self);
    }

    fn put_bit_set(&mut self, set: &BitSet) {
        self.put_var_int(&set.0);
        for b in set.1 {
            self.put_i64(*b);
        }
    }

    fn put_option<G>(&mut self, val: &Option<G>, write: impl FnOnce(&mut Self, &G)) {
        self.put_bool(val.is_some());
        if let Some(v) = val {
            write(self, v)
        }
    }

    fn put_list<G>(&mut self, list: &[G], write: impl Fn(&mut Self, &G)) {
        self.put_var_int(&list.len().into());
        for v in list {
            write(self, v);
        }
    }

    fn put_varint_arr(&mut self, v: &[i32]) {
        self.put_list(v, |p, &v| p.put_var_int(&v.into()))
    }
}

#[cfg(test)]
mod test {
    use bytes::{Bytes, BytesMut};
    use serde::{Deserialize, Serialize};

    use crate::{
        bytebuf::{deserializer, serializer},
        VarInt,
    };

    #[test]
    fn test_i32_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: i32,
        }
        let foo = Foo { bar: 69 };
        let mut serializer = serializer::Serializer::new(BytesMut::new());
        foo.serialize(&mut serializer).unwrap();

        let serialized: BytesMut = serializer.into();
        let deserialized: Foo = Foo::deserialize(deserializer::Deserializer::new(
            &mut Bytes::from(serialized),
        ))
        .unwrap();

        assert_eq!(foo, deserialized);
    }

    #[test]
    fn test_varint_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: VarInt,
        }
        let foo = Foo { bar: 69.into() };
        let mut serializer = serializer::Serializer::new(BytesMut::new());
        foo.serialize(&mut serializer).unwrap();

        let serialized: BytesMut = serializer.into();
        let deserialized: Foo = Foo::deserialize(deserializer::Deserializer::new(
            &mut Bytes::from(serialized),
        ))
        .unwrap();

        assert_eq!(foo, deserialized);
    }
}
