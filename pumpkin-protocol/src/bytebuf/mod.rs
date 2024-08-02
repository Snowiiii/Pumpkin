use core::str;
use std::io::{self, Error, ErrorKind};

use bytes::{Buf, BufMut, BytesMut};

use crate::{VarInt, VarLong};

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

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

    pub fn get_var_int(&mut self) -> VarInt {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        loop {
            let read = self.buffer.get_u8();

            value |= ((read & SEGMENT_BITS) as i32) << position;

            if read & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                panic!("VarInt is too big");
            }
        }

        value
    }

    pub fn get_var_long(&mut self) -> VarLong {
        let mut value: i64 = 0;
        let mut position: i64 = 0;

        loop {
            let read = self.buffer.get_u8();

            value |= ((read & SEGMENT_BITS) as i64) << position;

            if read & CONTINUE_BIT == 0 {
                break;
            }

            position += 7;

            if position >= 64 {
                panic!("VarInt is too big");
            }
        }

        value
    }

    pub fn get_string(&mut self) -> Result<String, io::Error> {
        self.get_string_len(32767)
    }

    pub fn get_string_len(&mut self, max_size: usize) -> Result<String, io::Error> {
        let size = self.get_var_int();
        if size as usize > max_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "String length is bigger than max size",
            ));
        }
        let data = self.buffer.copy_to_bytes(size as usize);
        if data.len() > max_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "String is bigger than max size",
            ));
        }
        match str::from_utf8(&data) {
            Ok(string_result) => Ok(string_result.to_string()),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }

    pub fn get_bool(&mut self) -> bool {
        self.buffer.get_u8() != 0
    }

    pub fn get_uuid(&mut self) -> uuid::Uuid {
        let mut bytes = [0u8; 16];
        self.buffer.copy_to_slice(&mut bytes);
        uuid::Uuid::from_slice(&bytes).expect("Failed to parse UUID")
    }

    pub fn put_bool(&mut self, v: bool) {
        if v {
            self.buffer.put_u8(1);
        } else {
            self.buffer.put_u8(0);
        }
    }

    pub fn put_uuid(&mut self, v: uuid::Uuid) {
        self.buffer.put_slice(v.as_bytes());
    }

    pub fn put_string(&mut self, val: &str) {
        self.put_var_int(val.len() as VarInt);
        self.buffer.put(val.as_bytes());
    }

    pub fn put_string_array(&mut self, array: &[String]) {
        for string in array {
            self.put_string(string)
        }
    }

    pub fn put_var_int(&mut self, value: VarInt) {
        let mut val = value as u32;
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

    /// Reads a boolean. If true, the closure is called, and the returned value is
    /// wrapped in Some. Otherwise, this returns None.
    pub fn get_option<T>(&mut self, val: impl FnOnce(&mut Self) -> T) -> Option<T> {
        if self.get_bool() {
            Some(val(self))
        } else {
            None
        }
    }
    /// Writes `true` if the option is Some, or `false` if None. If the option is
    /// some, then it also calls the `write` closure.
    pub fn put_option<T>(&mut self, val: &Option<T>, write: impl FnOnce(&mut Self, &T)) {
        self.put_bool(val.is_some());
        match val {
            Some(v) => write(self, v),
            None => {}
        }
    }

    pub fn get_list<T>(&mut self, val: impl Fn(&mut Self) -> T) -> Vec<T> {
        let len = self.get_var_int().try_into().unwrap();
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(val(self));
        }
        list
    }
    /// Writes a list to the buffer.
    pub fn put_list<T>(&mut self, list: &[T], write: impl Fn(&mut Self, &T)) {
        self.put_var_int(list.len().try_into().unwrap());
        for v in list {
            write(self, v);
        }
    }

    pub fn put_varint_arr(&mut self, v: &[i32]) {
        self.put_list(v, |p, &v| p.put_var_int(v))
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
}

// trait
impl ByteBuffer {
    pub fn get_u8(&mut self) -> u8 {
        self.buffer.get_u8()
    }

    pub fn get_i8(&mut self) -> i8 {
        self.buffer.get_i8()
    }

    pub fn get_u16(&mut self) -> u16 {
        self.buffer.get_u16()
    }

    pub fn get_i16(&mut self) -> i16 {
        self.buffer.get_i16()
    }

    pub fn get_u32(&mut self) -> u32 {
        self.buffer.get_u32()
    }

    pub fn get_i32(&mut self) -> i32 {
        self.buffer.get_i32()
    }

    pub fn get_u64(&mut self) -> u64 {
        self.buffer.get_u64()
    }

    pub fn get_i64(&mut self) -> i64 {
        self.buffer.get_i64()
    }

    pub fn get_f32(&mut self) -> f32 {
        self.buffer.get_f32()
    }

    pub fn get_f64(&mut self) -> f64 {
        self.buffer.get_f64()
    }

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

    pub fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
        self.buffer.copy_to_bytes(len)
    }
    pub fn put_slice(&mut self, src: &[u8]) {
        self.buffer.put_slice(src)
    }

    pub fn get_slice(&mut self) -> BytesMut {
        self.buffer.split()
    }
}
