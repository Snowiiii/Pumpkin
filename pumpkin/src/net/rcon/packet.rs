use std::{
    io::{BufRead, Cursor},
    string::FromUtf8Error,
};

use bytes::{BufMut, BytesMut};
use thiserror::Error;
use tokio::io::AsyncReadExt;

/// Client -> Server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ServerboundPacket {
    /// Typically, the first packet sent by the client, which is used to authenticate the connection with the server.
    Auth = 2,
    /// This packet type represents a command issued to the server by a client. This can be a `ConCommand` such as kill <player> or weather clear.
    /// The response will vary depending on the command issued.
    ExecCommand = 3,
}

impl ServerboundPacket {
    pub const fn from_i32(n: i32) -> Self {
        match n {
            //  3 => Self::Auth,
            2 => Self::ExecCommand,
            _ => Self::Auth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
/// Server -> Client
pub enum ClientboundPacket {
    /// This packet is a notification of the connection's current auth status. When the server receives an auth request, it will respond with an empty `SERVERDATA_RESPONSE_VALUE`,
    /// followed immediately by a `SERVERDATA_AUTH_RESPONSE` indicating whether authentication succeeded or failed. Note that the status code is returned in the packet id field, so when pairing the response with the original auth request, you may need to look at the packet id of the preceeding `SERVERDATA_RESPONSE_VALUE`.
    AuthResponse = 2,
    /// A `SERVERDATA_RESPONSE` packet is the response to a `SERVERDATA_EXECCOMMAND` request.
    Output = 0,
}

impl ClientboundPacket {
    pub fn write_buf(self, id: i32, body: &str) -> BytesMut {
        // let len = outgoing.len() as u64;
        let mut buf = BytesMut::new();
        // 10 is for 4 bytes ty, 4 bytes id, and 2 terminating nul bytes.
        buf.put_i32_le(10 + body.len() as i32);
        buf.put_i32_le(id);
        buf.put_i32_le(self as i32);
        let bytes = body.as_bytes();
        buf.put_slice(bytes);
        buf.put_u8(0);
        buf.put_u8(0);
        buf
    }
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("invalid length")]
    InvalidLength,
    #[error("failed to read packet")]
    FailedRead(std::io::Error),
    #[error("failed to send packet")]
    FailedSend(std::io::Error),
    #[error("invalid Packet String body")]
    InvalidBody(FromUtf8Error),
}

#[derive(Debug)]
/// Serverbound Packet
pub struct Packet {
    id: i32,
    ptype: ServerboundPacket,
    body: String,
}

impl Packet {
    pub async fn deserialize(incoming: &mut Vec<u8>) -> Result<Option<Self>, PacketError> {
        if incoming.len() < 4 {
            return Ok(None);
        }
        let mut buf = Cursor::new(&incoming);
        let len = buf.read_i32_le().await.map_err(PacketError::FailedRead)? + 4;
        if !(0..=1460).contains(&len) {
            return Err(PacketError::InvalidLength);
        }
        let id = buf.read_i32_le().await.map_err(PacketError::FailedRead)?;
        let ty = buf.read_i32_le().await.map_err(PacketError::FailedRead)?;
        let mut payload = vec![];
        let _ = buf
            .read_until(b'\0', &mut payload)
            .map_err(PacketError::FailedRead)?;
        payload.pop();
        buf.read_u8().await.map_err(PacketError::FailedRead)?;
        if buf.position() != len as u64 {
            return Err(PacketError::InvalidLength);
        }
        incoming.drain(0..len as usize);

        let packet = Self {
            id,
            ptype: ServerboundPacket::from_i32(ty),
            body: String::from_utf8(payload).map_err(PacketError::InvalidBody)?,
        };

        Ok(Some(packet))
    }
    pub fn get_body(&self) -> &str {
        &self.body
    }

    pub const fn get_type(&self) -> ServerboundPacket {
        self.ptype
    }

    pub const fn get_id(&self) -> i32 {
        self.id
    }
}
