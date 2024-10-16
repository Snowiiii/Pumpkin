use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serializer,
};

struct NetworkSerializer<T: bytes::BufMut>(T);

/// Many of the primitive types can be serialized with a single function call
macro_rules! fn_serialize_primitive {
    ($ty:ty, $serialize:ident => $put_fn:ident) => {
        fn $serialize(self, v: $ty) -> Result<Self::Ok, Self::Error> {
            self.0.$put_fn(v);
            Ok(self)
        }
    };
}

/// Quick macro to generate the unimplemented functions for the serializer
macro_rules! fn_unimplemented {
    ($ty:ty,$fn:ident => $description:expr) => {
        fn $fn(self, _: $ty) -> Result<Self::Ok, Self::Error> {
            unimplemented!($description)
        }
    };
    ($fn:ident => $description:expr) => {
        fn $fn(self) -> Result<Self::Ok, Self::Error> {
            unimplemented!($description)
        }
    };
}

impl<'a, Buf: bytes::BufMut> Serializer for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.put_u8(v as u8);
        Ok(self)
    }

    fn_serialize_primitive!(i8, serialize_i8  => put_i8);
    fn_serialize_primitive!(i16,serialize_i16 => put_i16);
    fn_serialize_primitive!(i32,serialize_i32 => put_i32);
    fn_serialize_primitive!(i64,serialize_i64 => put_i64);
    fn_serialize_primitive!(u8, serialize_u8  => put_u8);
    fn_serialize_primitive!(u16,serialize_u16 => put_u16);
    fn_serialize_primitive!(u32,serialize_u32 => put_u32);
    fn_serialize_primitive!(u64,serialize_u64 => put_u64);
    fn_serialize_primitive!(f32,serialize_f32 => put_f32);
    fn_serialize_primitive!(f64,serialize_f64 => put_f64);

    fn_unimplemented!(char,serialize_char => "char is not supported");
    fn_unimplemented!(&str,serialize_str => "bare strings are not supported");
    fn_unimplemented!(&str,serialize_unit_struct => "Unit structs are not supported without specialization");
    fn_unimplemented!(serialize_unit => "Unit types are not supported without specialization");

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_some<T>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        unimplemented!()
    }
}

impl<'a, Buf> SerializeSeq for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        T::serialize(value, &mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, Buf> SerializeTuple for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        T::serialize(value, &mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

impl<'a, Buf> SerializeTupleStruct for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        T::serialize(value, &mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

impl<'a, Buf> SerializeTupleVariant for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        T::serialize(value, &mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

impl<'a, Buf> SerializeMap for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_key<T>(&mut self, _: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        unimplemented!("Maps are not supported by the network format directly")
    }

    fn serialize_value<T>(&mut self, _: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        unimplemented!("Maps are not supported byt he network format directly");
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!("Maps are not supported byt he network format directly");
    }
}

impl<'a, Buf> SerializeStruct for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        T::serialize(value, &mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, Buf> SerializeStructVariant for &'a mut NetworkSerializer<Buf>
where
    Buf: bytes::BufMut,
{
    type Ok = Self;

    type Error = super::PacketError;

    fn serialize_field<T>(&mut self, _: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        T::serialize(value, &mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
