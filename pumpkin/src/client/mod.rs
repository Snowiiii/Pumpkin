use std::{
    collections::VecDeque,
    io::{self, Write},
    rc::Rc,
};

use crate::{
    protocol::{
        server::{
            handshake::SHandShake,
            status::{SPingRequest, SStatusRequest},
        },
        ClientPacket, RawPacket,
    },
    server::Server,
};

use crate::protocol::{bytebuf::buffer::ByteBuffer, ConnectionState};
use mio::{event::Event, net::TcpStream, Registry};
use std::io::Read;

mod client_packet;
pub mod player;

use client_packet::ClientPacketProcessor;

pub struct Client {
    pub server: Rc<Server>,
    pub connection_state: ConnectionState,
    pub closed: bool,
    pub connection: TcpStream,
    // Packets coming from the client -> server
    client_packets_queue: VecDeque<RawPacket>,
}

impl Client {
    pub fn new(server: Rc<Server>, connection: TcpStream) -> Self {
        Self {
            server,
            connection_state: ConnectionState::HandShake,
            connection,
            closed: false,
            client_packets_queue: VecDeque::new(),
        }
    }

    pub fn add_packet(&mut self, packet: RawPacket) {
        self.client_packets_queue.push_back(packet);
    }

    pub fn send_packet<P: ClientPacket>(&mut self, packet: P) {
        dbg!("WRITING");
        let mut packet_buf = ByteBuffer::new();
        packet.write(&mut packet_buf);
        // Creating empty packet's buffer
        let mut packet = ByteBuffer::new();
        // Creating length's bytes buffer and fill it as VarInt
        let mut len_bytes = ByteBuffer::new();
        len_bytes.write_var_int(P::PACKET_ID);
        // Writing full packet's length(content + length's bytes)
        packet.write_var_int((packet_buf.len() + len_bytes.len()) as i32);
        // Writing length bytes
        packet.write_all(len_bytes.as_bytes()).unwrap();
        // Drop(Free) length bytes buffer
        drop(len_bytes);
        // Writing some packet's content
        packet.write_all(packet_buf.as_bytes()).unwrap();

        self.connection.write_all(packet.as_bytes()).unwrap();
        self.connection.flush().unwrap(); // todo do not flush every time
    }

    pub fn procress_packets(&mut self) {
        let mut i = 0;
        while i < self.client_packets_queue.len() {
            let mut packet = self.client_packets_queue.remove(i).unwrap();
            self.handle_packet(&mut packet);
            i += 1;
        }
    }

    pub fn handle_packet(&mut self, packet: &mut RawPacket) {
        dbg!("Handling packet");
        let bytebuf = &mut packet.bytebuf;
        match self.connection_state {
            crate::protocol::ConnectionState::HandShake => match packet.id {
                SHandShake::PACKET_ID => self.handle_handshake(SHandShake::read(bytebuf)),
                _ => log::error!(
                    "Failed to handle packet id {} while in Handshake state",
                    packet.id
                ),
            },
            crate::protocol::ConnectionState::Status => match packet.id {
                SStatusRequest::PACKET_ID => {
                    self.handle_status_request(SStatusRequest::read(bytebuf))
                }
                SPingRequest::PACKET_ID => self.handle_ping_request(SPingRequest::read(bytebuf)),
                _ => log::error!(
                    "Failed to handle packet id {} while in Status state",
                    packet.id
                ),
            },
            _ => log::error!("Invalid Connection state {:?}", self.connection_state),
        }
    }

    /// Returns `true` if the connection is done.
    pub fn handle_connection_event(
        &mut self,
        _registry: &Registry,
        event: &Event,
    ) -> io::Result<()> {
        if event.is_readable() {
            let mut received_data = vec![0; 4096];
            let mut bytes_read = 0;
            // We can (maybe) read from the connection.
            loop {
                match self.connection.read(&mut received_data[bytes_read..]) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        self.closed = true;
                        break;
                    }
                    Ok(n) => {
                        bytes_read += n;
                        if bytes_read == received_data.len() {
                            received_data.resize(received_data.len() + 1024, 0);
                        }
                    }
                    // Would block "errors" are the OS's way of saying that the
                    // connection is not actually ready to perform this I/O operation.
                    Err(ref err) if would_block(err) => break,
                    Err(ref err) if interrupted(err) => continue,
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
            }

            if bytes_read != 0 {
                let received_data = &received_data[..bytes_read];
                let mut bytebuf = ByteBuffer::from_bytes(received_data);
                let packet = RawPacket {
                    len: bytebuf.read_var_int().unwrap(),
                    id: bytebuf.read_var_int().unwrap(),
                    bytebuf,
                };
                if packet.len + 1 != received_data.len() as i32 {
                    log::error!(
                        "Packet length does not match Data length: {} {}",
                        packet.len,
                        received_data.len()
                    );
                }
                dbg!(&packet);
                self.add_packet(packet);
                self.procress_packets();
            }
        }
        Ok(())
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

pub fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
