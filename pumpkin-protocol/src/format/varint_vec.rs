use std::{fmt, marker::PhantomData};

use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::varint::VarInt;

/**
 * A serialization format for a varint-prefixed vector.
 */
struct OwnedVarIntVec<T> {
    data: Vec<T>,
}

struct VarIntVec<'a, T> {
    data: &'a Vec<T>,
}

pub mod vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{OwnedVarIntVec, VarIntVec};

    pub fn serialize<S, T>(data: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        VarIntVec { data }.serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let vec = OwnedVarIntVec::deserialize(deserializer)?;
        Ok(vec.data)
    }
}

impl<'de, T> Deserialize<'de> for OwnedVarIntVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarIntVecVisitor<T>(PhantomData<T>);
        impl<'de, T> Visitor<'de> for VarIntVecVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple of a varint and a vector")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let len: VarInt = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let len = len.into();
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    vec.push(
                        seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                    );
                }
                Ok(vec)
            }
        }
        deserializer
            .deserialize_tuple(2, VarIntVecVisitor(PhantomData))
            .map(|data| Self { data })
    }
}

impl<'a, T> Serialize for VarIntVec<'a, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&VarInt::from(self.data.len()))?;
        tup.serialize_element(&self.data)?;
        tup.end()
    }
}
