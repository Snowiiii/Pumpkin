use bytes::BufMut;
use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{BitSet, ClientPacket, ServerPacket, VarInt, VarIntType};

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
        let mut value = self.0 as u32;
        let mut buf = Vec::new();

        while value > 0x7F {
            buf.put_u8(value as u8 | 0x80);
            value >>= 7;
        }

        buf.put_u8(value as u8);

        serializer.serialize_bytes(&buf)
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
                let mut val = 0;
                for i in 0..VarInt::MAX_SIZE {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        val |= (i32::from(byte) & 0b01111111) << (i * 7);
                        if byte & 0b10000000 == 0 {
                            return Ok(VarInt(val));
                        }
                    } else {
                        break;
                    }
                }
                Err(de::Error::custom("VarInt was too large"))
            }
        }

        deserializer.deserialize_seq(VarIntVisitor)
    }
}

pub trait ClientPacketID {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: ClientPacketID + Serialize,
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
    P: DeserializeOwned,
{
    fn read(bytebuf: &mut ByteBuffer) -> Result<P, DeserializerError> {
        let deserializer = deserializer::Deserializer::new(bytebuf);
        P::deserialize(deserializer)
    }
}
