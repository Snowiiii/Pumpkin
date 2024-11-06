use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{BitSet, ClientPacket, ServerPacket, VarEncodedInteger, VarInt, VarIntType, VarLong};

use super::{deserializer, serializer, ByteBuffer, DeserializerError};

impl<'a> Serialize for BitSet<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: make this right
        (&self.0, self.1).serialize(serializer)
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.encode(|buff| serializer.serialize_bytes(buff))
    }
}

impl<'de> Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarIntVisitor;

        impl<'de> Visitor<'de> for VarIntVisitor {
            type Value = VarInt;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                VarInt::try_decode(|| seq.next_element::<u8>(), de::Error::custom)
            }
        }

        deserializer.deserialize_seq(VarIntVisitor)
    }
}

impl Serialize for VarLong {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.encode(|buff| serializer.serialize_bytes(buff))
    }
}

impl<'de> Deserialize<'de> for VarLong {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarLongVisitor;

        impl<'de> Visitor<'de> for VarLongVisitor {
            type Value = VarLong;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                VarLong::try_decode(|| seq.next_element::<u8>(), de::Error::custom)
            }
        }

        deserializer.deserialize_seq(VarLongVisitor)
    }
}

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write(&self, bytebuf: &mut ByteBuffer) {
        let mut serializer = serializer::Serializer::new(ByteBuffer::empty());
        self.serialize(&mut serializer)
            .expect("Could not serialize packet");
        // We write the packet in an empty bytebuffer and then put it into our current one.
        // In the future we may do packet batching thats the reason i don't let every packet create a new bytebuffer and use
        // an existing instead
        bytebuf.put(serializer.output.buf());
    }
}

impl<P> ServerPacket for P
where
    P: Packet + DeserializeOwned,
{
    fn read(bytebuf: &mut ByteBuffer) -> Result<P, DeserializerError> {
        let deserializer = deserializer::Deserializer::new(bytebuf);
        P::deserialize(deserializer)
    }
}
