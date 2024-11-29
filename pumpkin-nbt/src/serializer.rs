use bytes::{BufMut, BytesMut};
use serde::ser::Impossible;
use serde::{ser, Serialize};
use std::io::Write;

use crate::tag::NbtTag;
use crate::{
    Error, BYTE_ARRAY_ID, BYTE_ID, COMPOUND_ID, DOUBLE_ID, END_ID, FLOAT_ID, INT_ARRAY_ID, INT_ID,
    LIST_ID, LONG_ARRAY_ID, LONG_ID, SHORT_ID, STRING_ID,
};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Serializer {
    output: BytesMut,
    state: State,
}

// NBT has a different order of things, then most other formats
// So I use State, to keep what serializer has to do, and some information like field name
#[derive(Clone, Debug, PartialEq)]
enum State {
    // In network NBT root name is not present
    Root(Option<String>),
    Named(String),
    // Used by maps, to check if key is String
    MapKey,
    FirstListElement { len: i32 },
    ListElement,
    Array { name: String, array_type: String },
}

impl Serializer {
    fn parse_state(&mut self, tag: u8) -> Result<()> {
        match &mut self.state {
            State::Named(name) | State::Array { name, .. } => {
                self.output.put_u8(tag);
                self.output
                    .put(NbtTag::String(name.clone()).serialize_data());
            }
            State::FirstListElement { len } => {
                self.output.put_u8(tag);
                self.output.put_i32(*len);
            }
            State::MapKey => {
                if tag != STRING_ID {
                    return Err(Error::SerdeError(format!(
                        "Map key can only be string, not {tag}"
                    )));
                }
            }
            State::ListElement => {}
            _ => return Err(Error::SerdeError("Invalid Serializer state!".to_string())),
        };
        Ok(())
    }
}

/// Serializes struct using Serde Serializer to unnamed (network) NBT
pub fn to_bytes_unnamed<T>(value: &T) -> Result<BytesMut>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: BytesMut::new(),
        state: State::Root(None),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn to_writer_unnamed<T, W>(value: &T, mut writer: W) -> Result<()>
where
    T: Serialize,
    W: Write,
{
    writer.write_all(&to_bytes_unnamed(value)?).unwrap();
    Ok(())
}

/// Serializes struct using Serde Serializer to normal NBT
pub fn to_bytes<T>(value: &T, name: String) -> Result<BytesMut>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: BytesMut::new(),
        state: State::Root(Some(name)),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn to_writer<T, W>(value: &T, name: String, mut writer: W) -> Result<()>
where
    T: Serialize,
    W: Write,
{
    writer.write_all(&to_bytes(value, name)?).unwrap();
    Ok(())
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    // NBT doesn't have bool type, but it's most commonly represented as a byte
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_i8(v as i8)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.parse_state(BYTE_ID)?;
        self.output.put_i8(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.parse_state(SHORT_ID)?;
        self.output.put_i16(v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.parse_state(INT_ID)?;
        self.output.put_i32(v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.parse_state(LONG_ID)?;
        self.output.put_i64(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.parse_state(BYTE_ID)?;
        self.output.put_u8(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.parse_state(SHORT_ID)?;
        self.output.put_u16(v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.parse_state(INT_ID)?;
        self.output.put_u32(v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.parse_state(LONG_ID)?;
        self.output.put_u64(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.parse_state(FLOAT_ID)?;
        self.output.put_f32(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.parse_state(DOUBLE_ID)?;
        self.output.put_f64(v);
        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::UnsupportedType("char".to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.parse_state(STRING_ID)?;
        if self.state == State::MapKey {
            self.state = State::Named(v.to_string());
            return Ok(());
        }

        self.output
            .put(NbtTag::String(v.to_string()).serialize_data());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.parse_state(LIST_ID)?;
        self.output.put_u8(BYTE_ID);
        self.output.put_i32(v.len() as i32);
        self.output.put_slice(v);
        Ok(())
    }

    // Just skip serializing, if value is none
    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::UnsupportedType("unit struct".to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("newtype struct".to_string()))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name != "nbt_array" {
            return Err(Error::SerdeError(
                "new_type variant supports only nbt_array".to_string(),
            ));
        }

        let name = match self.state {
            State::Named(ref name) => name.clone(),
            _ => return Err(Error::SerdeError("Invalid Serializer state!".to_string())),
        };

        self.state = State::Array {
            name,
            array_type: variant.to_string(),
        };

        value.serialize(self)?;

        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len.is_none() {
            return Err(Error::SerdeError(
                "Length of the sequence must be known first!".to_string(),
            ));
        }

        match &mut self.state {
            State::Array { array_type, .. } => {
                let id = match array_type.as_str() {
                    "byte" => BYTE_ARRAY_ID,
                    "int" => INT_ARRAY_ID,
                    "long" => LONG_ARRAY_ID,
                    _ => {
                        return Err(Error::SerdeError(
                            "Array supports only byte, int, long".to_string(),
                        ))
                    }
                };
                self.parse_state(id)?;
                self.output.put_i32(len.unwrap() as i32);
                self.state = State::ListElement;
            }
            _ => {
                self.parse_state(LIST_ID)?;
                self.state = State::FirstListElement {
                    len: len.unwrap() as i32,
                };
            }
        }

        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType("tuple".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType("tuple variant".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        if let State::FirstListElement { .. } = self.state {
            self.parse_state(COMPOUND_ID)?;
        } else if let State::ListElement = self.state {
            return Ok(self);
        } else {
            self.output.put_u8(COMPOUND_ID);
        }
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.output.put_u8(COMPOUND_ID);

        match &mut self.state {
            State::Root(root_name) => {
                if let Some(root_name) = root_name {
                    self.output
                        .put(NbtTag::String(root_name.clone()).serialize_data());
                }
            }
            State::Named(string) => {
                self.output
                    .put(NbtTag::String(string.clone()).serialize_data());
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType("struct variant".to_string()))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.state = State::ListElement;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.state = State::Named(key.to_string());
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.put_u8(END_ID);
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.state = State::MapKey;
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output.put_u8(END_ID);
        Ok(())
    }
}
