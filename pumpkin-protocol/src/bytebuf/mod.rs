use crate::{BitSet, FixedBitSet, VarInt, VarLongType};
use bytes::{Buf, BufMut, BytesMut};
use core::str;

mod deserializer;
pub use deserializer::DeserializerError;
pub mod packet_id;
mod serializer;

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

#[derive(Debug)]
pub struct ByteBuffer {
    buffer: BytesMut,
}

impl ByteBuffer {
    pub fn empty() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }
    pub fn new(buffer: BytesMut) -> Self {
        Self { buffer }
    }

    pub fn get_var_int(&mut self) -> Result<VarInt, DeserializerError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        loop {
            let read = self.get_u8()?;

            value |= ((read & SEGMENT_BITS) as i32) << position;

            if read & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                return Err(DeserializerError::Message("VarInt is too big".to_string()));
            }
        }

        Ok(VarInt(value))
    }

    pub fn get_var_long(&mut self) -> Result<VarLongType, DeserializerError> {
        let mut value: i64 = 0;
        let mut position: i64 = 0;

        loop {
            let read = self.get_u8()?;

            value |= ((read & SEGMENT_BITS) as i64) << position;

            if read & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 64 {
                return Err(DeserializerError::Message("VarLong is too big".to_string()));
            }
        }

        Ok(value)
    }

    pub fn get_string(&mut self) -> Result<String, DeserializerError> {
        self.get_string_len(i16::MAX as i32)
    }

    pub fn get_string_len(&mut self, max_size: i32) -> Result<String, DeserializerError> {
        let size = self.get_var_int()?.0;
        if size > max_size {
            return Err(DeserializerError::Message(
                "String length is bigger than max size".to_string(),
            ));
        }

        let data = self.copy_to_bytes(size as usize)?;
        if data.len() as i32 > max_size {
            return Err(DeserializerError::Message(
                "String is bigger than max size".to_string(),
            ));
        }
        match str::from_utf8(&data) {
            Ok(string_result) => Ok(string_result.to_string()),
            Err(e) => Err(DeserializerError::Message(e.to_string())),
        }
    }

    pub fn get_bool(&mut self) -> Result<bool, DeserializerError> {
        Ok(self.get_u8()? != 0)
    }

    pub fn get_uuid(&mut self) -> Result<uuid::Uuid, DeserializerError> {
        let mut bytes = [0u8; 16];
        self.copy_to_slice(&mut bytes)?;
        // Prefer from_bytes instead from_slice
        Ok(uuid::Uuid::from_bytes(bytes))
    }

    pub fn get_fixed_bitset(&mut self, bits: usize) -> Result<FixedBitSet, DeserializerError> {
        self.copy_to_bytes(bits.div_ceil(8))
    }

    pub fn put_bool(&mut self, v: bool) {
        if v {
            self.buffer.put_u8(1);
        } else {
            self.buffer.put_u8(0);
        }
    }

    pub fn put_uuid(&mut self, v: &uuid::Uuid) {
        // thats the vanilla way
        let pair = v.as_u64_pair();
        self.put_u64(pair.0);
        self.put_u64(pair.1);
    }

    pub fn put_string(&mut self, val: &str) {
        self.put_string_len(val, i16::MAX as i32);
    }

    pub fn put_string_len(&mut self, val: &str, max_size: i32) {
        if val.len() as i32 > max_size {
            // Should be panic?, I mean its our fault
            panic!("String is too big");
        }
        self.put_var_int(&val.len().into());
        self.buffer.put(val.as_bytes());
    }

    pub fn put_string_array(&mut self, array: &[String]) {
        for string in array {
            self.put_string(string)
        }
    }

    pub fn put_var_int(&mut self, value: &VarInt) {
        let mut val = value.0;
        for _ in 0..5 {
            let mut b: u8 = val as u8 & 0b01111111;
            val >>= 7;
            if val != 0 {
                b |= 0b10000000;
            }
            self.buffer.put_u8(b);
            if val == 0 {
                break;
            }
        }
    }

    pub fn put_bit_set(&mut self, set: &BitSet) {
        self.put_var_int(&set.0);
        for b in set.1 {
            self.put_i64(*b);
        }
    }

    /// Reads a boolean. If true, the closure is called, and the returned value is
    /// wrapped in Some. Otherwise, this returns None.
    pub fn get_option<T>(
        &mut self,
        val: impl FnOnce(&mut Self) -> Result<T, DeserializerError>,
    ) -> Result<Option<T>, DeserializerError> {
        if self.get_bool()? {
            Ok(Some(val(self)?))
        } else {
            Ok(None)
        }
    }
    /// Writes `true` if the option is Some, or `false` if None. If the option is
    /// some, then it also calls the `write` closure.
    pub fn put_option<T>(&mut self, val: &Option<T>, write: impl FnOnce(&mut Self, &T)) {
        self.put_bool(val.is_some());
        if let Some(v) = val {
            write(self, v)
        }
    }

    pub fn get_list<T>(
        &mut self,
        val: impl Fn(&mut Self) -> Result<T, DeserializerError>,
    ) -> Result<Vec<T>, DeserializerError> {
        let len = self.get_var_int()?.0 as usize;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(val(self)?);
        }
        Ok(list)
    }
    /// Writes a list to the buffer.
    pub fn put_list<T>(&mut self, list: &[T], write: impl Fn(&mut Self, &T)) {
        self.put_var_int(&list.len().into());
        for v in list {
            write(self, v);
        }
    }

    pub fn put_varint_arr(&mut self, v: &[i32]) {
        self.put_list(v, |p, &v| p.put_var_int(&v.into()))
    }

    /*  pub fn get_nbt(&mut self) -> Option<fastnbt::value::Value> {
            match crab_nbt::NbtTag::deserialize(self.buf()) {
                Ok(v) => Some(v),
                Err(err) => None,
            }
        }

        pub fn put_nbt(&mut self, nbt: N) {
            self.buffer.put_slice(&nbt.serialize());
        }
    */
    pub fn buf(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    // Trait equivalents
    pub fn get_u8(&mut self) -> Result<u8, DeserializerError> {
        if self.buffer.has_remaining() {
            Ok(self.buffer.get_u8())
        } else {
            Err(DeserializerError::Message(
                "No bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_i8(&mut self) -> Result<i8, DeserializerError> {
        if self.buffer.has_remaining() {
            Ok(self.buffer.get_i8())
        } else {
            Err(DeserializerError::Message(
                "No bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_u16(&mut self) -> Result<u16, DeserializerError> {
        if self.buffer.remaining() >= 2 {
            Ok(self.buffer.get_u16())
        } else {
            Err(DeserializerError::Message(
                "Less than 2 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_i16(&mut self) -> Result<i16, DeserializerError> {
        if self.buffer.remaining() >= 2 {
            Ok(self.buffer.get_i16())
        } else {
            Err(DeserializerError::Message(
                "Less than 2 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_u32(&mut self) -> Result<u32, DeserializerError> {
        if self.buffer.remaining() >= 4 {
            Ok(self.buffer.get_u32())
        } else {
            Err(DeserializerError::Message(
                "Less than 4 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_i32(&mut self) -> Result<i32, DeserializerError> {
        if self.buffer.remaining() >= 4 {
            Ok(self.buffer.get_i32())
        } else {
            Err(DeserializerError::Message(
                "Less than 4 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_u64(&mut self) -> Result<u64, DeserializerError> {
        if self.buffer.remaining() >= 8 {
            Ok(self.buffer.get_u64())
        } else {
            Err(DeserializerError::Message(
                "Less than 8 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_i64(&mut self) -> Result<i64, DeserializerError> {
        if self.buffer.remaining() >= 8 {
            Ok(self.buffer.get_i64())
        } else {
            Err(DeserializerError::Message(
                "Less than 8 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_f32(&mut self) -> Result<f32, DeserializerError> {
        if self.buffer.remaining() >= 4 {
            Ok(self.buffer.get_f32())
        } else {
            Err(DeserializerError::Message(
                "Less than 4 bytes left to consume".to_string(),
            ))
        }
    }

    pub fn get_f64(&mut self) -> Result<f64, DeserializerError> {
        if self.buffer.remaining() >= 8 {
            Ok(self.buffer.get_f64())
        } else {
            Err(DeserializerError::Message(
                "Less than 8 bytes left to consume".to_string(),
            ))
        }
    }

    // TODO: SerializerError?
    pub fn put_u8(&mut self, n: u8) {
        self.buffer.put_u8(n)
    }

    pub fn put_i8(&mut self, n: i8) {
        self.buffer.put_i8(n)
    }

    pub fn put_u16(&mut self, n: u16) {
        self.buffer.put_u16(n)
    }

    pub fn put_i16(&mut self, n: i16) {
        self.buffer.put_i16(n)
    }

    pub fn put_u32(&mut self, n: u32) {
        self.buffer.put_u32(n)
    }

    pub fn put_i32(&mut self, n: i32) {
        self.buffer.put_i32(n)
    }

    pub fn put_u64(&mut self, n: u64) {
        self.buffer.put_u64(n)
    }

    pub fn put_i64(&mut self, n: i64) {
        self.buffer.put_i64(n)
    }

    pub fn put_f32(&mut self, n: f32) {
        self.buffer.put_f32(n)
    }

    pub fn put_f64(&mut self, n: f64) {
        self.buffer.put_f64(n)
    }

    pub fn copy_to_bytes(&mut self, len: usize) -> Result<bytes::Bytes, DeserializerError> {
        if self.buffer.len() >= len {
            Ok(self.buffer.copy_to_bytes(len))
        } else {
            Err(DeserializerError::Message(
                "Unable to copy bytes".to_string(),
            ))
        }
    }

    pub fn copy_to_slice(&mut self, dst: &mut [u8]) -> Result<(), DeserializerError> {
        if self.buffer.remaining() >= dst.len() {
            self.buffer.copy_to_slice(dst);
            Ok(())
        } else {
            Err(DeserializerError::Message(
                "Unable to copy slice".to_string(),
            ))
        }
    }

    pub fn put_slice(&mut self, src: &[u8]) {
        self.buffer.put_slice(src)
    }

    pub fn put<T: Buf>(&mut self, src: T)
    where
        Self: Sized,
    {
        self.buffer.put(src)
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional)
    }

    pub fn get_slice(&mut self) -> BytesMut {
        self.buffer.split()
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::{
        bytebuf::{deserializer, serializer, ByteBuffer},
        VarInt,
    };

    #[test]
    fn test_i32_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: i32,
        }
        let foo = Foo { bar: 69 };
        let mut serializer = serializer::Serializer::new(ByteBuffer::empty());
        foo.serialize(&mut serializer).unwrap();

        let mut serialized: ByteBuffer = serializer.into();
        let deserialized: Foo =
            Foo::deserialize(deserializer::Deserializer::new(&mut serialized)).unwrap();

        assert_eq!(foo, deserialized);
    }

    #[test]
    fn test_varint_reserialize() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            bar: VarInt,
        }
        let foo = Foo { bar: 69.into() };
        let mut serializer = serializer::Serializer::new(ByteBuffer::empty());
        foo.serialize(&mut serializer).unwrap();

        let mut serialized: ByteBuffer = serializer.into();
        let deserialized: Foo =
            Foo::deserialize(deserializer::Deserializer::new(&mut serialized)).unwrap();

        assert_eq!(foo, deserialized);
    }
}
