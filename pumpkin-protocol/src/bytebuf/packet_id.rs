use serde::{
    de::{self, DeserializeOwned, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{BitSet, ClientPacket, ServerPacket, VarInt, VarIntType};

use super::{deserializer, serializer, ByteBuffer, DeserializerError};

impl Serialize for BitSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: make this right
        (self.0.clone(), self.1.clone()).serialize(serializer)
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut val = self.0;
        let mut buf: Vec<u8> = Vec::new();
        for _ in 0..5 {
            let mut b: u8 = val as u8 & 0b01111111;
            val >>= 7;
            if val != 0 {
                b |= 0b10000000;
            }
            buf.push(b);
            if val == 0 {
                break;
            }
        }
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

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write(&self, bytebuf: &mut ByteBuffer) {
        take_mut::take(bytebuf, |bytebuf| {
            let mut serializer = serializer::Serializer::new(bytebuf);
            self.serialize(&mut serializer)
                .expect("Could not serialize packet");
            serializer.into()
        });
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
