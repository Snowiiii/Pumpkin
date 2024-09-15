use std::io::{self, BufRead, Cursor, Write};

use bytes::{BufMut, BytesMut};
use thiserror::Error;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    Auth,
    AuthResponse,
    ExecCommand,
    Output,
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("invalid length")]
    InvalidLength,
    #[error("expected terminating NUL byte")]
    NoNullTermination,
    #[error("wrong password")]
    WrongPassword,
}

impl PacketType {
    fn to_i32(self) -> i32 {
        match self {
            PacketType::Auth => 3,
            PacketType::AuthResponse => 2,
            PacketType::ExecCommand => 2,
            PacketType::Output => 0,
        }
    }

    pub fn from_i32(n: i32) -> PacketType {
        match n {
            3 => PacketType::Auth,
            2 => PacketType::ExecCommand,
            _ => PacketType::Output,
        }
    }
}

#[derive(Debug)]
pub struct Packet {
    id: i32,
    ptype: PacketType,
    body: String,
}

impl Packet {
    pub fn new(id: i32, ptype: PacketType, body: String) -> Packet {
        Packet { id, ptype, body }
    }

    pub async fn send_packet(&mut self, connection: &mut TcpStream) -> io::Result<()> {
        // let len = outgoing.len() as u64;
        let mut buf = BytesMut::new();
        // 10 is for 4 bytes ty, 4 bytes id, and 2 terminating nul bytes.
        buf.put_i32_le(10 + self.get_body().len() as i32);
        buf.put_i32_le(self.id);
        buf.put_i32_le(self.get_type().to_i32());
        let bytes = self.get_body().as_bytes();
        buf.put_slice(bytes);
        buf.put_u8(0);
        buf.put_u8(0);
        let _ = connection.write(&buf).await.unwrap();
        Ok(())
    }

    pub async fn deserialize(incoming: &mut Vec<u8>) -> Result<Option<Packet>, PacketError> {
        if incoming.len() < 4 {
            return Ok(None);
        }
        let mut buf = Cursor::new(&incoming);
        let len = buf.read_i32_le().await.unwrap() + 4;
        if !(0..=1460).contains(&len) {
            return Err(PacketError::InvalidLength);
        }
        let id = buf.read_i32_le().await.unwrap();
        let ty = buf.read_i32_le().await.unwrap();
        let mut payload = vec![];
        let _ = buf.read_until(b'\0', &mut payload).unwrap();
        payload.pop();
        if buf.read_u8().await.unwrap() != 0 {
            return Err(PacketError::NoNullTermination);
        }
        if buf.position() != len as u64 {
            return Err(PacketError::InvalidLength);
        }
        incoming.drain(0..len as usize);

        let packet = Packet {
            id,
            ptype: PacketType::from_i32(ty),
            body: String::from_utf8(payload).unwrap(),
        };

        Ok(Some(packet))
    }
    pub fn get_body(&self) -> &str {
        &self.body
    }

    pub fn get_type(&self) -> PacketType {
        self.ptype
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
}
