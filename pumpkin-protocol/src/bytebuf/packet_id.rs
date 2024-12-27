use bytes::{Buf, BufMut};
use serde::{de::DeserializeOwned, Serialize};

use crate::{codec::var_int::VarIntType, ClientPacket, ServerPacket};

use super::{deserializer, serializer, ReadingError};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write(&self, bytebuf: &mut impl BufMut) {
        let mut serializer = serializer::Serializer::new(bytebuf);
        self.serialize(&mut serializer)
            .expect("Could not serialize packet");
    }
}

impl<P> ServerPacket for P
where
    P: Packet + DeserializeOwned,
{
    fn read(bytebuf: &mut impl Buf) -> Result<P, ReadingError> {
        let deserializer = deserializer::Deserializer::new(bytebuf);
        P::deserialize(deserializer)
    }
}
