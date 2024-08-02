use std::{
    collections::VecDeque,
    io::{self, Write},
    rc::Rc,
};

use crate::{
    entity::player::{ChatMode, Hand, Player},
    server::Server,
};

use mio::{event::Event, net::TcpStream, Token};
use pumpkin_protocol::{
    client::{config::CConfigDisconnect, login::CLoginDisconnect},
    packet_decoder::PacketDecoder,
    packet_encoder::PacketEncoder,
    server::{
        config::{SAcknowledgeFinishConfig, SClientInformation, SKnownPacks, SPluginMessage},
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
        status::{SPingRequest, SStatusRequest},
    },
    ClientPacket, ConnectionState, PacketError, RawPacket, ServerPacket,
};

use rsa::Pkcs1v15Encrypt;
use std::io::Read;
use thiserror::Error;

mod client_packet;
mod player_packet;

use client_packet::ClientPacketProcessor;

pub struct PlayerConfig {
    locale: String, // 16
    view_distance: i8,
    chat_mode: ChatMode,
    chat_colors: bool,
    skin_parts: u8,
    main_hand: Hand,
    text_filtering: bool,
    server_listing: bool,
}

pub struct Client {
    pub player: Option<Player>,

    pub name: Option<String>,
    pub uuid: Option<uuid::Uuid>,
    pub config: Option<PlayerConfig>,
    pub brand: Option<String>,

    pub connection_state: ConnectionState,
    pub encrytion: bool,
    pub closed: bool,
    pub token: Rc<Token>,
    pub connection: TcpStream,
    enc: PacketEncoder,
    dec: PacketDecoder,
    pub client_packets_queue: VecDeque<RawPacket>,
}

impl Client {
    pub fn new(token: Rc<Token>, connection: TcpStream) -> Self {
        Self {
            name: None,
            uuid: None,
            config: None,
            brand: None,
            token,
            player: None,
            connection_state: ConnectionState::HandShake,
            connection,
            enc: PacketEncoder::default(),
            dec: PacketDecoder::default(),
            encrytion: true,
            closed: false,
            client_packets_queue: VecDeque::new(),
        }
    }

    /// adds a Incoming packet to the queue
    pub fn add_packet(&mut self, packet: RawPacket) {
        self.client_packets_queue.push_back(packet);
    }

    /// enables encryption
    pub fn enable_encryption(
        &mut self,
        server: &mut Server,
        shared_secret: Vec<u8>,
    ) -> Result<(), EncryptionError> {
        self.encrytion = true;
        let shared_secret = server
            .private_key
            .decrypt(Pkcs1v15Encrypt, &shared_secret)
            .map_err(|_| EncryptionError::FailedDecrypt)?;
        let crypt_key: [u8; 16] = shared_secret
            .as_slice()
            .try_into()
            .map_err(|_| EncryptionError::SharedWrongLength)?;
        self.dec.enable_encryption(&crypt_key);
        self.enc.enable_encryption(&crypt_key);
        Ok(())
    }

    /// Im many cases we want to kick the Client when an Packet Error occours, But especially in the Client state we will not try to kick when not important packets
    /// e.g Postion, Rotation... has not been send
    pub fn send_packet<P: ClientPacket>(&mut self, packet: P) -> Result<(), PacketError> {
        dbg!("sending packet");
        self.enc.append_packet(packet)?;
        self.connection
            .write_all(&self.enc.take())
            .map_err(|_| PacketError::ConnectionWrite)?;
        Ok(())
    }

    pub fn process_packets(&mut self, server: &mut Server) {
        let mut i = 0;
        while i < self.client_packets_queue.len() {
            let mut packet = self.client_packets_queue.remove(i).unwrap();
            self.handle_packet(server, &mut packet);
            i += 1;
        }
    }

    /// Handles an incoming decoded Packet
    pub fn handle_packet(&mut self, server: &mut Server, packet: &mut RawPacket) {
        dbg!("Handling packet");
        let bytebuf = &mut packet.bytebuf;
        match self.connection_state {
            pumpkin_protocol::ConnectionState::HandShake => match packet.id {
                SHandShake::PACKET_ID => self.handle_handshake(server, SHandShake::read(bytebuf)),
                _ => log::error!(
                    "Failed to handle packet id {} while in Handshake state",
                    packet.id
                ),
            },
            pumpkin_protocol::ConnectionState::Status => match packet.id {
                SStatusRequest::PACKET_ID => {
                    self.handle_status_request(server, SStatusRequest::read(bytebuf))
                }
                SPingRequest::PACKET_ID => {
                    self.handle_ping_request(server, SPingRequest::read(bytebuf))
                }
                _ => log::error!(
                    "Failed to handle packet id {} while in Status state",
                    packet.id
                ),
            },
            pumpkin_protocol::ConnectionState::Login => match packet.id {
                SLoginStart::PACKET_ID => {
                    self.handle_login_start(server, SLoginStart::read(bytebuf))
                }
                SEncryptionResponse::PACKET_ID => {
                    self.handle_encryption_response(server, SEncryptionResponse::read(bytebuf))
                }
                SLoginPluginResponse::PACKET_ID => {
                    self.handle_plugin_response(server, SLoginPluginResponse::read(bytebuf))
                }
                SLoginAcknowledged::PACKET_ID => {
                    self.handle_login_acknowledged(server, SLoginAcknowledged::read(bytebuf))
                }
                _ => log::error!(
                    "Failed to handle packet id {} while in Login state",
                    packet.id
                ),
            },
            pumpkin_protocol::ConnectionState::Config => match packet.id {
                SClientInformation::PACKET_ID => {
                    self.handle_client_information(server, SClientInformation::read(bytebuf))
                }
                SPluginMessage::PACKET_ID => {
                    self.handle_plugin_message(server, SPluginMessage::read(bytebuf))
                }
                SAcknowledgeFinishConfig::PACKET_ID => {
                    self.handle_config_acknowledged(server, SAcknowledgeFinishConfig::read(bytebuf))
                }
                SKnownPacks::PACKET_ID => {
                    self.handle_known_packs(server, SKnownPacks::read(bytebuf))
                }
                _ => log::error!(
                    "Failed to handle packet id {} while in Config state",
                    packet.id
                ),
            },
            _ => log::error!("Invalid Connection state {:?}", self.connection_state),
        }
    }

    // Reads the connection until our buffer of len 4096 is full, then decode
    /// Close connection when an error occurs
    pub fn poll(&mut self, server: &mut Server, event: &Event) {
        if event.is_readable() {
            let mut received_data = vec![0; 4096];
            let mut bytes_read = 0;
            // We can (maybe) read from the connection.
            loop {
                match self.connection.read(&mut received_data[bytes_read..]) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        self.close();
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
                    Err(_) => self.close(),
                }
            }

            if bytes_read != 0 {
                self.dec.reserve(4096);
                self.dec.queue_slice(&received_data[..bytes_read]);
                match self.dec.decode() {
                    Ok(packet) => {
                        if let Some(packet) = packet {
                            self.add_packet(packet);
                            self.process_packets(server);
                        }
                    }
                    Err(err) => self.kick(&err.to_string()),
                }
                self.dec.clear();
            }
        }
    }

    /// Kicks the Client with a reason depending on the connection state
    pub fn kick(&mut self, reason: &str) {
        // Todo
        match self.connection_state {
            ConnectionState::Login => {
                self.send_packet(CLoginDisconnect::new(reason))
                    .unwrap_or_else(|_| self.close());
            }
            ConnectionState::Config => {
                self.send_packet(CConfigDisconnect::new(reason))
                    .unwrap_or_else(|_| self.close());
            }
            _ => {
                log::warn!("Cant't kick in {:?} State", self.connection_state)
            }
        }
        self.close()
    }

    /// You should prefer to use `kick` when you can
    pub fn close(&mut self) {
        self.closed = true;
    }
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("failed to decrypt shared secret")]
    FailedDecrypt,
    #[error("shared secret has the wrong length")]
    SharedWrongLength,
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

pub fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
