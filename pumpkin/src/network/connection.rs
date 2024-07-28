use std::{
    io::{self, Read},
    string::FromUtf8Error,
    sync::Arc,
};

use crossbeam_channel::{Receiver, Sender};
use mio::{net::TcpStream, Token, Waker};

use crate::player::JoinInfo;

use super::{
    protocol::{clientbound, read::MessageReader, serverbound},
    ProtocolVersion,
};

pub struct Connection {
    stream: TcpStream,
    ver: Option<ProtocolVersion>,

    tx: Sender<clientbound::Packet>,
    rx: Receiver<clientbound::Packet>,

    token: Token,

    incoming: Vec<u8>,
    outgoing: Vec<u8>,
    garbage: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ConnSender {
    tx: Sender<cb::Packet>,
    wake: Sender<WakeEvent>,
    waker: Arc<Waker>,
    tok: Token,
}

pub struct NewConn {
    pub sender: ConnSender,
    pub info: JoinInfo,
}

#[derive(Debug)]
enum ParseError {
    InvalidType(i32),
    CannotHandleOutput,
    InvalidLength,
    NotLoggedIn,
    AlreadyLoggedIn,
    InvalidPassword,
    IO(io::Error),
    InvalidMessage(FromUtf8Error),
}

impl Connection {
    fn new(stream: TcpStream, token: Token) -> Self {
        // For a 10 chunk render distance, we need to send 441 packets at once. So a
        // limit of 512 means we don't block very much.
        let (tx, rx) = crossbeam_channel::bounded(512);
        Self {
            stream,
            rx,
            tx,
            token,
            incoming: Vec::with_capacity(1024),
            outgoing: Vec::with_capacity(1024),
            garbage: vec![0; 256 * 1024],
        }
    }
    pub fn send(&self, p: clientbound::Packet) {
        if let Ok(()) = self.tx.send(p) {}
    }

    pub fn read(&mut self) -> io::Result<(bool, Option<NewConn>, Vec<serverbound::Packet>)> {
        let mut out = vec![];
        loop {
            let n = match self.stream.read(&mut self.garbage) {
                Ok(0) => return Ok((true, None, out)),
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok((false, None, out)),
                Err(e) => return Err(e),
            };
            self.incoming.extend_from_slice(&self.garbage[..n]);
            let (new_conn, packets) = self.read_incoming()?;
            if new_conn.is_some() {
                return Ok((false, new_conn, packets));
            }
            out.extend(packets);
        }
    }

    fn read_incoming(&mut self) -> io::Result<(Option<NewConn>, Vec<serverbound::Packet>)> {
        let mut out = vec![];
        while !self.incoming.is_empty() {
            let mut m = MessageReader::new(&self.incoming);
            match m.read_u32() {
                Ok(len) => {
                    let len = len as usize;
                    if len + m.index() <= self.incoming.len() {
                        // Remove the length varint at the start
                        let idx = m.index();
                        self.incoming.drain(0..idx);
                        // We already handshaked
                        if self.ver.is_some() {
                            let mut m = MessageReader::new(&self.incoming[..len]);
                            let p = serverbound::Packet::read(&mut m).map_err(|err| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("while reading packet got err: {err}"),
                                )
                            })?;
                            let n = m.index();
                            self.incoming.drain(0..n);
                            if n != len {
                                return Err(io::Error::new(
                      io::ErrorKind::InvalidData,
                      format!("packet did not parse enough bytes (expected {len}, only parsed {n})"),
                    ));
                            }
                            out.push(p);
                        } else {
                            // This is the first packet, so it must be a login packet.
                            let mut m = MessageReader::new(&self.incoming[..len]);
                            let info: JoinInfo = m.read().map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("error reading handshake: {e}"),
                                )
                            })?;
                            let n = m.index();
                            self.incoming.drain(0..n);
                            if n != len {
                                return Err(io::Error::new(
                      io::ErrorKind::InvalidData,
                      format!("handshake did not parse enough bytes (expected {len}, only parsed {n})"),
                    ));
                            }
                            self.ver = Some(ProtocolVersion::from(info.ver as i32));
                            // We rely on the caller to set the player using this value.
                            return Ok((
                                Some(NewConn {
                                    sender: self.sender(),
                                    info,
                                }),
                                out,
                            ));
                        }
                    } else {
                        break;
                    }
                }
                // If this is an EOF, then we have a partial varint, so we are done reading.
                Err(e) => {
                    if matches!(e, ReadError::Invalid(InvalidReadError::EOF)) {
                        return Ok((None, out));
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("error reading packet id: {e}"),
                        ));
                    }
                }
            }
        }
        Ok((None, out))
    }
}
