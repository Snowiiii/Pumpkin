use std::{
    io::{self, Cursor, Write},
    ops::Deref,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use compound::NbtCompound;
use tag::NbtTag;
use thiserror::Error;

pub mod compound;
pub mod serializer;
pub mod tag;

pub const END_ID: u8 = 0;
pub const BYTE_ID: u8 = 1;
pub const SHORT_ID: u8 = 2;
pub const INT_ID: u8 = 3;
pub const LONG_ID: u8 = 4;
pub const FLOAT_ID: u8 = 5;
pub const DOUBLE_ID: u8 = 6;
pub const BYTE_ARRAY_ID: u8 = 7;
pub const STRING_ID: u8 = 8;
pub const LIST_ID: u8 = 9;
pub const COMPOUND_ID: u8 = 10;
pub const INT_ARRAY_ID: u8 = 11;
pub const LONG_ARRAY_ID: u8 = 12;

#[derive(Error, Debug)]
pub enum ReadingError {
    #[error("The root tag of the NBT file is not a compound tag. Received tag id: {0}")]
    NoRootCompound(u8),
    #[error("Encountered an unknown NBT tag id {0}.")]
    UnknownTagId(u8),
    #[error("Failed to Cesu 8 Decode")]
    Cesu8DecodingError,
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Nbt {
    pub name: String,
    pub root_tag: NbtCompound,
}

impl Nbt {
    pub fn new(name: String, tag: NbtCompound) -> Self {
        Nbt {
            name,
            root_tag: tag,
        }
    }

    pub fn read(bytes: &mut impl Buf) -> Result<Nbt, ReadingError> {
        let tag_type_id = bytes.get_u8();

        if tag_type_id != COMPOUND_ID {
            return Err(ReadingError::NoRootCompound(tag_type_id));
        }

        Ok(Nbt {
            name: get_nbt_string(bytes)?,
            root_tag: NbtCompound::deserialize_content(bytes)?,
        })
    }

    pub fn read_from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Nbt, ReadingError> {
        Self::read(cursor)
    }

    /// Reads NBT tag, that doesn't contain the name of root compound.
    /// Used in [Network NBT](https://wiki.vg/NBT#Network_NBT_(Java_Edition)).
    pub fn read_unnamed(bytes: &mut impl Buf) -> Result<Nbt, ReadingError> {
        let tag_type_id = bytes.get_u8();

        if tag_type_id != COMPOUND_ID {
            return Err(ReadingError::NoRootCompound(tag_type_id));
        }

        Ok(Nbt {
            name: String::new(),
            root_tag: NbtCompound::deserialize_content(bytes)
                .map_err(|_| ReadingError::Cesu8DecodingError)?,
        })
    }

    pub fn read_unnamed_from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Nbt, ReadingError> {
        Self::read_unnamed(cursor)
    }

    pub fn write(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put_u8(COMPOUND_ID);
        bytes.put(NbtTag::String(self.name.to_string()).serialize_data());
        bytes.put(self.root_tag.serialize_content());
        bytes.freeze()
    }

    pub fn write_to_writer<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write())?;
        Ok(())
    }

    /// Writes NBT tag, without name of root compound.
    /// Used in [Network NBT](https://wiki.vg/NBT#Network_NBT_(Java_Edition)).
    pub fn write_unnamed(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put_u8(COMPOUND_ID);
        bytes.put(self.root_tag.serialize_content());
        bytes.freeze()
    }

    pub fn write_unnamed_to_writer<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.write_unnamed())?;
        Ok(())
    }
}

impl Deref for Nbt {
    type Target = NbtCompound;

    fn deref(&self) -> &Self::Target {
        &self.root_tag
    }
}

impl From<NbtCompound> for Nbt {
    fn from(value: NbtCompound) -> Self {
        Nbt::new(String::new(), value)
    }
}

impl<T> AsRef<T> for Nbt
where
    T: ?Sized,
    <Nbt as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

impl AsMut<NbtCompound> for Nbt {
    fn as_mut(&mut self) -> &mut NbtCompound {
        &mut self.root_tag
    }
}

pub fn get_nbt_string(bytes: &mut impl Buf) -> Result<String, ReadingError> {
    let len = bytes.get_u16() as usize;
    let string_bytes = bytes.copy_to_bytes(len);
    let string =
        cesu8::from_java_cesu8(&string_bytes).map_err(|_| ReadingError::Cesu8DecodingError)?;
    Ok(string.to_string())
}
