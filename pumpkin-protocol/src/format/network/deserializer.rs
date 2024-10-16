use serde::de::{self, SeqAccess};

struct NetworkDeserializer<'de, T>
where
    T: bytes::Buf + 'de,
{
    buffer: &'de mut T,
}

impl<'de, T> NetworkDeserializer<'de, T>
where
    T: bytes::Buf + 'de,
{
    pub fn new(buffer: &'de mut T) -> Self {
        Self { buffer }
    }
}

/// Deserializes a primitive type from the buffer as a raw value.
macro_rules! fn_deserialize_primitive {
    ($deserialize_fn:ident, $visit_fn:ident, $get_fn:ident) => {
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit_fn(self.buffer.$get_fn())
        }
    };
}

// Quick macro to generate the unimplemented functions for the deserializer
macro_rules! fn_unimplemented {
    ($fn:ident => $description:expr) => {
        fn $fn<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            unimplemented!($description)
        }
    };
    ($fn:ident) => {
        fn_unimplemented!($fn => "This function is not implemented for the network format")
    };
    () => {

    };
}

// Using concat_idents! to generate the deserialize functions for the primitive types.
// Requires nightly `concat_idents`.
// macro_rules! impl_primitives {
//     ($($ty:ty),*) => {
//         $(
//             fn_deserialize_primitive!(concat_idents!(deserialize_, $ty), concat_idents!(visit_, $ty), concat_idents!(get_, $ty));
//         )*
//     }
// }

impl<'de, 'a, T> de::Deserializer<'de> for &'a mut NetworkDeserializer<'de, T>
where
    T: bytes::Buf + 'de,
{
    type Error = super::PacketError;

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccessor(&mut self))
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccessor(&mut self))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(self.buffer.get_u8() != 0)
    }

    // impl_primitives!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
    fn_deserialize_primitive!(deserialize_i8, visit_i8, get_i8);
    fn_deserialize_primitive!(deserialize_u8, visit_u8, get_u8);
    fn_deserialize_primitive!(deserialize_i16, visit_i16, get_i16);
    fn_deserialize_primitive!(deserialize_u16, visit_u16, get_u16);
    fn_deserialize_primitive!(deserialize_i32, visit_i32, get_i32);
    fn_deserialize_primitive!(deserialize_u32, visit_u32, get_u32);
    fn_deserialize_primitive!(deserialize_i64, visit_i64, get_i64);
    fn_deserialize_primitive!(deserialize_u64, visit_u64, get_u64);
    fn_deserialize_primitive!(deserialize_f32, visit_f32, get_f32);
    fn_deserialize_primitive!(deserialize_f64, visit_f64, get_f64);

    fn_unimplemented!(deserialize_any => "The network format is not self describing, so it's impossible to infer the next type");
    fn_unimplemented!(deserialize_char => "Chars as defined by Rust are not a network primitive");
    fn_unimplemented!(deserialize_str => "Strings are UTF-encoded strings prefixed by a VarInt length, but the maximum length is context-dependent, thus is not implemented in the general case");
    fn_unimplemented!(deserialize_string => "Strings are UTF-encoded prefixed by a VarInt length, but the maximum length is context-dependent, thus cannot be implemented in the general case");
    fn_unimplemented!(deserialize_bytes => "As the network format is not self-describing, it's impossible to infer the length of a byte array in the general case");
    fn_unimplemented!(deserialize_byte_buf => "As the network format is not self-describing, it's impossible to infer the length of a byte array in the general case");
    fn_unimplemented!(deserialize_ignored_any => "Unit values are not a network primitive.");
    fn_unimplemented!(deserialize_option => "Whether a value is present requires more context than can be inferred in the general case");
    fn_unimplemented!(deserialize_unit => "Unit values are not a network primitive");
    fn_unimplemented!(deserialize_identifier => "Identifiers are not a network primitive");
    fn_unimplemented!(deserialize_map => "Maps are not a network primitive");

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(&mut *self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!("Enums are not a network primitive. Look at implementing a specialization which implements (de)serialize")
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Unit structs are not a network primitive")
    }
}

struct SeqAccessor<'a, 'de: 'a, T>(&'a mut NetworkDeserializer<'de, T>)
where
    T: bytes::Buf + 'de;

impl<'a, 'de, Buf> SeqAccess<'de> for SeqAccessor<'a, 'de, Buf>
where
    Buf: bytes::Buf + 'de,
{
    type Error = super::PacketError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.0).map(Some)
    }
}
