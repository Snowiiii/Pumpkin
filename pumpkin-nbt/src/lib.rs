use std::{
    fmt::Display,
    io::{self, Cursor, Write},
    ops::Deref,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use cesu8::Cesu8DecodingError;
use compound::NbtCompound;
use serde::{de, ser};
use serde::{Deserialize, Deserializer};
use tag::NbtTag;
use thiserror::Error;

pub mod compound;
pub mod deserializer;
pub mod serializer;
pub mod tag;

// This NBT crate is inspired from CrabNBT

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
pub enum Error {
    #[error("The root tag of the NBT file is not a compound tag. Received tag id: {0}")]
    NoRootCompound(u8),
    #[error("Encountered an unknown NBT tag id {0}.")]
    UnknownTagId(u8),
    #[error("Failed to Cesu 8 Decode")]
    Cesu8DecodingError,
    #[error("Serde error: {0}")]
    SerdeError(String),
    #[error("NBT doesn't support this type {0}")]
    UnsupportedType(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
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

    pub fn read(bytes: &mut impl Buf) -> Result<Nbt, Error> {
        let tag_type_id = bytes.get_u8();

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Nbt {
            name: get_nbt_string(bytes).map_err(|_| Error::Cesu8DecodingError)?,
            root_tag: NbtCompound::deserialize_content(bytes)?,
        })
    }

    pub fn read_from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Nbt, Error> {
        Self::read(cursor)
    }

    /// Reads NBT tag, that doesn't contain the name of root compound.
    pub fn read_unnamed(bytes: &mut impl Buf) -> Result<Nbt, Error> {
        let tag_type_id = bytes.get_u8();

        if tag_type_id != COMPOUND_ID {
            return Err(Error::NoRootCompound(tag_type_id));
        }

        Ok(Nbt {
            name: String::new(),
            root_tag: NbtCompound::deserialize_content(bytes)
                .map_err(|_| Error::Cesu8DecodingError)?,
        })
    }

    pub fn read_unnamed_from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Nbt, Error> {
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

pub fn get_nbt_string(bytes: &mut impl Buf) -> Result<String, Cesu8DecodingError> {
    let len = bytes.get_u16() as usize;
    let string_bytes = bytes.copy_to_bytes(len);
    let string = cesu8::from_java_cesu8(&string_bytes)?;
    Ok(string.to_string())
}

macro_rules! impl_array {
    ($name:ident, $variant:expr) => {
        pub struct $name;

        impl $name {
            pub fn serialize<T, S>(input: T, serializer: S) -> Result<S::Ok, S::Error>
            where
                T: serde::Serialize,
                S: serde::Serializer,
            {
                serializer.serialize_newtype_variant("nbt_array", 0, $variant, &input)
            }

            pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
            where
                T: Deserialize<'de>,
                D: Deserializer<'de>,
            {
                T::deserialize(deserializer)
            }
        }
    };
}

impl_array!(IntArray, "int");
impl_array!(LongArray, "long");
impl_array!(BytesArray, "byte");

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::BytesArray;
    use crate::IntArray;
    use crate::LongArray;
    use crate::{
        deserializer::from_bytes_unnamed,
        serializer::to_bytes_unnamed,
    };

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        byte: i8,
        short: i16,
        int: i32,
        long: i64,
        float: f32,
        string: String,
    }

    #[test]
    fn test_simple_ser_de_unamed() {
        let test = Test {
            byte: 123,
            short: 1342,
            int: 4313,
            long: 34,
            float: 1.00,
            string: "Hello test".to_string(),
        };
        let mut bytes = to_bytes_unnamed(&test).unwrap();
        let recreated_struct: Test = from_bytes_unnamed(&mut bytes).unwrap();

        assert_eq!(test, recreated_struct);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestArray {
        #[serde(with = "BytesArray")]
        byte_array: Vec<u8>,
        #[serde(with = "IntArray")]
        int_array: Vec<i32>,
        #[serde(with = "LongArray")]
        long_array: Vec<i64>,
    }

    #[test]
    fn test_simple_ser_de_array() {
        let test = TestArray {
            byte_array: vec![0, 3, 2],
            int_array: vec![13, 1321, 2],
            long_array: vec![1, 0, 200301, 1],
        };
        let mut bytes = to_bytes_unnamed(&test).unwrap();
        let recreated_struct: TestArray = from_bytes_unnamed(&mut bytes).unwrap();

        assert_eq!(test, recreated_struct);
    }
}
