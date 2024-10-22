use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use super::varint::VarInt;

/// A length prefixed [Java BitSet](https://docs.oracle.com/javase/7/docs/api/java/util/BitSet.html)
pub struct BitSet(Vec<i64>);

impl Serialize for BitSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (VarInt::from(self.0.len()), &self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BitSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BitSetVisitor;
        impl<'de> Visitor<'de> for BitSetVisitor {
            type Value = Vec<i64>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a varint-prefixed list of bits")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let len: usize = seq
                    .next_element::<VarInt>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?
                    .into();
                let mut data = Vec::with_capacity(len);
                for _ in 0..len {
                    data.push(seq.next_element::<i64>()?.unwrap());
                }
                Ok(data)
            }
        }
        let data = deserializer.deserialize_seq(BitSetVisitor)?;
        Ok(BitSet(data))
    }
}
