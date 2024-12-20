use bytes::{BufMut, Bytes, BytesMut};
use serde::{de::DeserializeOwned, Serialize};

use crate::{ClientPacket, ServerPacket, VarIntType};

use super::{deserializer, serializer, ReadingError};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write(&self, bytebuf: &mut BytesMut) {
        let mut serializer = serializer::Serializer::new(BytesMut::new());
        self.serialize(&mut serializer)
            .expect("Could not serialize packet");
        // We write the packet in an empty bytebuffer and then put it into our current one.
        // In the future we may do packet batching thats the reason i don't let every packet create a new bytebuffer and use
        // an existing instead
        bytebuf.put(serializer.output);
    }
}

impl<P> ServerPacket for P
where
    P: Packet + DeserializeOwned,
{
    fn read(bytebuf: &mut Bytes) -> Result<P, ReadingError> {
        let deserializer = deserializer::Deserializer::new(bytebuf);
        P::deserialize(deserializer)
    }
}
