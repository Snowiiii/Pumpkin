use std::collections::HashMap;

use error::Error;
use error::Result;
use nbt::Tag;
use nbt::NBT;
use serde::{ser, Serialize};

mod deserialize;
mod error;
pub mod nbt;
mod serialize;

pub fn to_nbt<T>(name: &str, value: &T) -> anyhow::Result<NBT>
where
    T: Serialize,
{
    Ok(NBT::new(name, to_tag(value)?))
}

pub fn to_tag<T>(value: &T) -> anyhow::Result<Tag>
where
    T: Serialize,
{
    let mut serializer = Serializer { tag: Tag::End };
    value.serialize(&mut serializer)?;
    Ok(serializer.tag)
}

pub struct Serializer {
    tag: Tag,
}

pub struct SeqSerializer<'a> {
    ser: &'a mut Serializer,
    items: Vec<Tag>,
}
pub struct MapSerializer<'a> {
    ser: &'a mut Serializer,
    key: Option<String>,
    items: HashMap<String, Tag>,
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = SeqSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = MapSerializer<'a>;
    type SerializeStructVariant = MapSerializer<'a>;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.tag = Tag::Byte(v as i8);
        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.tag = Tag::Byte(v);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.tag = Tag::Short(v);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.tag = Tag::Int(v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.tag = Tag::Long(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_i8(v as i8)
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i16(v as i16)
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_i32(v as i32)
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.tag = Tag::Float(v);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.tag = Tag::Double(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }
    fn serialize_str(self, v: &str) -> Result<()> {
        self.tag = Tag::String(v.into());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.tag = Tag::ByteArr(v.into());
        Ok(())
    }

    // There isn't really a `None` in NBT. There is `Tag::End`, which could be
    // searched and removed if this was in a struct or map, which would essentially
    // skip this value if it's `None`. However, I don't care enough, so I'm just
    // going to produce an error.
    fn serialize_none(self) -> Result<()> {
        Err(Error::CannotSerializeNone)
    }
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::CannotSerializeNone)
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            ser: self,
            items: vec![],
        })
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            ser: self,
            key: None,
            items: HashMap::new(),
        })
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    // We don't support enums (they don't really make sense)
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::Enum)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Enum)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Enum)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Enum)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.ser)?;
        let tag = std::mem::replace(&mut self.ser.tag, Tag::End);
        if let Some(first) = self.items.first() {
            let expected_ty = first.ty();
            let actual_ty = tag.ty();
            if expected_ty != actual_ty {
                return Err(Error::ListType(first.clone(), tag));
            }
        }
        self.items.push(tag);
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.tag = Tag::List(self.items);
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.ser)?;
        let tag = std::mem::replace(&mut self.ser.tag, Tag::End);
        self.key = match tag {
            Tag::String(key) => Some(key),
            other => return Err(Error::MapKey(other)),
        };
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.ser)?;
        let tag = std::mem::replace(&mut self.ser.tag, Tag::End);
        self.items.insert(self.key.take().unwrap(), tag);
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.tag = self.items.into();
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.ser)?;
        let tag = std::mem::replace(&mut self.ser.tag, Tag::End);
        self.items.insert(key.into(), tag);
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.tag = self.items.into();
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.ser)?;
        let tag = std::mem::replace(&mut self.ser.tag, Tag::End);
        self.items.insert(key.into(), tag);
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.tag = self.items.into();
        Ok(())
    }
}
