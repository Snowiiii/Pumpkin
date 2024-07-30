use flate2::read::{GzDecoder, ZlibDecoder};
use std::{collections::HashMap, error::Error, fmt, io, io::Read, string::FromUtf8Error};

use crate::protocol::bytebuf::buffer::ByteBuffer;

use super::{nbt::ParseError, Tag, NBT};

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(ty) => write!(f, "invalid tag type: {ty}"),
            Self::InvalidString(e) => write!(f, "invalid string: {e}"),
            Self::IO(e) => write!(f, "io error: {e}"),
        }
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(e: FromUtf8Error) -> ParseError {
        ParseError::InvalidString(e)
    }
}
impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> ParseError {
        ParseError::IO(e)
    }
}
impl Error for ParseError {}

impl NBT {
    pub fn deserialize_file(buf: Vec<u8>) -> Result<Self, ParseError> {
        if buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b {
            // This means its gzipped
            let mut d: GzDecoder<&[u8]> = GzDecoder::new(buf.as_ref());
            let mut buf = vec![];
            d.read_to_end(&mut buf)?;
            Self::deserialize(buf)
        } else {
            // It could be zlib compressed or not compressed
            let mut d: ZlibDecoder<&[u8]> = ZlibDecoder::new(buf.as_ref());
            let mut decompressed = vec![];
            match d.read_to_end(&mut decompressed) {
                Ok(_) => Self::deserialize(decompressed),
                Err(_) => Self::deserialize(buf),
            }
        }
    }
    /// Deserializes the given byte array as nbt data.
    pub fn deserialize(mut buf: Vec<u8>) -> Result<Self, ParseError> {
        Self::deserialize_buf(&mut ByteBuffer::from_vec(buf))
    }
    /// Deserializes the given buffer as nbt data. This will continue reading
    /// where this buffer is currently placed, and will advance the reader to be
    /// right after the nbt data. If this function returns an error, then the
    /// buffer will be in an undefined state (it will still be safe, but there are
    /// no guarantees as too how far ahead the buffer will have been advanced).
    pub fn deserialize_buf(buf: &mut ByteBuffer) -> Result<Self, ParseError> {
        let ty = buf.read_u8()?;
        if ty == 0 {
            Ok(NBT::empty())
        } else {
            let len = buf.read_u16()?;
            let name = String::from_utf8(buf.read_bytes(len as usize)?)?;
            Ok(NBT::new(&name, Tag::deserialize(ty, buf)?))
        }
    }
}

impl Tag {
    fn deserialize(ty: u8, buf: &mut ByteBuffer) -> Result<Self, ParseError> {
        match ty {
            0 => Ok(Self::End),
            1 => Ok(Self::Byte(buf.read_i8()?)),
            2 => Ok(Self::Short(buf.read_i16()?)),
            3 => Ok(Self::Int(buf.read_i32()?)),
            4 => Ok(Self::Long(buf.read_i64()?)),
            5 => Ok(Self::Float(buf.read_f32()?)),
            6 => Ok(Self::Double(buf.read_f64()?)),
            7 => {
                let len = buf.read_i32()?;
                Ok(Self::ByteArr(buf.read_bytes(len as usize)?))
            }
            8 => {
                let len = buf.read_u16()?;
                match String::from_utf8(buf.read_bytes(len as usize)?) {
                    Ok(v) => Ok(Self::String(v)),
                    Err(e) => Err(ParseError::InvalidString(e)),
                }
            }
            9 => {
                let inner_ty = buf.read_u8()?;
                let len = buf.read_i32()?;
                let mut inner = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    inner.push(Tag::deserialize(inner_ty, buf)?);
                }
                Ok(Self::List(inner))
            }
            10 => {
                let mut inner = HashMap::new();
                loop {
                    let ty = buf.read_u8()?;
                    if ty == Self::End.ty() {
                        break;
                    }
                    let len = buf.read_u16()?;
                    let name = String::from_utf8(buf.read_bytes(len as usize)?).unwrap();
                    let tag = Tag::deserialize(ty, buf)?;
                    inner.insert(name, tag);
                }
                Ok(inner.into())
            }
            11 => {
                let len = buf.read_i32()?;
                let mut inner = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    inner.push(buf.read_i32()?);
                }
                Ok(Self::IntArray(inner))
            }
            12 => {
                let len = buf.read_i32()?;
                let mut inner = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    inner.push(buf.read_i64()?);
                }
                Ok(Self::LongArray(inner))
            }
            _ => Err(ParseError::InvalidType(ty)),
        }
    }
}
