use serde::{Serialize, Serializer};

use crate::{ClientPacket, VarInt, VarInt32};

use super::{serializer, ByteBuffer};

impl Serialize for VarInt32 {
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

pub trait Packet {
    const PACKET_ID: VarInt;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write(&self, bytebuf: &mut ByteBuffer) {
        dbg!(P::PACKET_ID);
        take_mut::take(bytebuf, |bytebuf| {
            let mut serializer = serializer::Serializer::new(bytebuf);
            self.serialize(&mut serializer)
                .expect("Could not serialize packet");
            serializer.into()
        });
    }
}
