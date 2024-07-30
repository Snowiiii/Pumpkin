use std::{collections::HashMap, error::Error, fmt, string::FromUtf8Error};

use crate::protocol::bytebuf::buffer::ByteBuffer;

use super::{Tag, NBT};

#[derive(Debug)]
pub enum ParseError {
    InvalidType(u8),
    InvalidString(FromUtf8Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(ty) => write!(f, "invalid tag type: {}", ty),
            Self::InvalidString(e) => write!(f, "invalid string: {}", e),
        }
    }
}

impl Error for ParseError {}

impl NBT {
    /// Deserializes the given byte array as nbt data.
    pub fn deserialize(buf: Vec<u8>) -> Result<Self, ParseError> {
        Self::deserialize_buf(&mut ByteBuffer::from_vec(buf))
    }
    /// Deserializes the given buffer as nbt data. This will continue reading
    /// where this buffer is currently placed, and will advance the reader to be
    /// right after the nbt data. If this function returns an error, then the
    /// buffer will be in an undefined state (it will still be safe, but there are
    /// no guarantees as too how far ahead the buffer will have been advanced).
    pub fn deserialize_buf(buf: &mut ByteBuffer) -> Result<Self, ParseError> {
        let ty = buf.read_u8().unwrap();
        let len = buf.read_u16().unwrap();
        let name = String::from_utf8(buf.read_bytes(len as usize).unwrap()).unwrap();
        Ok(NBT::new(&name, Tag::deserialize(ty, buf)?))
    }
}

impl Tag {
    fn deserialize(ty: u8, buf: &mut ByteBuffer) -> Result<Self, ParseError> {
        match ty {
            0 => Ok(Self::End),
            1 => Ok(Self::Byte(buf.read_i8().unwrap())),
            2 => Ok(Self::Short(buf.read_i16().unwrap())),
            3 => Ok(Self::Int(buf.read_i32().unwrap())),
            4 => Ok(Self::Long(buf.read_i64().unwrap())),
            5 => Ok(Self::Float(buf.read_f32().unwrap())),
            6 => Ok(Self::Double(buf.read_f64().unwrap())),
            7 => {
                let len = buf.read_i32().unwrap();
                Ok(Self::ByteArr(buf.read_bytes(len as usize).unwrap()))
            }
            8 => {
                let len = buf.read_u16().unwrap();
                match String::from_utf8(buf.read_bytes(len as usize).unwrap()) {
                    Ok(v) => Ok(Self::String(v)),
                    Err(e) => Err(ParseError::InvalidString(e)),
                }
            }
            9 => {
                let inner_ty = buf.read_u8().unwrap();
                let len = buf.read_i32().unwrap();
                let mut inner = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    inner.push(Tag::deserialize(inner_ty, buf)?);
                }
                Ok(Self::List(inner))
            }
            10 => {
                let mut inner = HashMap::new();
                loop {
                    let ty = buf.read_u8().unwrap();
                    if ty == Self::End.ty() {
                        break;
                    }
                    let len = buf.read_u16().unwrap();
                    let name = String::from_utf8(buf.read_bytes(len as usize).unwrap()).unwrap();
                    let tag = Tag::deserialize(ty, buf)?;
                    inner.insert(name, tag);
                }
                Ok(Self::Compound(inner))
            }
            11 => {
                let len = buf.read_i32().unwrap();
                let mut inner = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    inner.push(buf.read_i32().unwrap());
                }
                Ok(Self::IntArray(inner))
            }
            12 => {
                let len = buf.read_i32().unwrap();
                let mut inner = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    inner.push(buf.read_i64().unwrap());
                }
                Ok(Self::LongArray(inner))
            }
            _ => Err(ParseError::InvalidType(ty)),
        }
    }
}
