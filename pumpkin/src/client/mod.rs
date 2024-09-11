use std::{
    io::{self, Write},
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicI32},
        Arc,
    },
};

use crate::{
    entity::player::{ChatMode, Hand},
    server::Server,
};

use authentication::GameProfile;
use crossbeam::atomic::AtomicCell;
use mio::{event::Event, net::TcpStream, Token};
use parking_lot::Mutex;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::{
    bytebuf::{packet_id::Packet, DeserializerError},
    client::{config::CConfigDisconnect, login::CLoginDisconnect, play::CPlayDisconnect},
    packet_decoder::PacketDecoder,
    packet_encoder::PacketEncoder,
    server::{
        config::{SAcknowledgeFinishConfig, SClientInformationConfig, SKnownPacks, SPluginMessage},
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
        status::{SStatusPingRequest, SStatusRequest},
    },
    ClientPacket, ConnectionState, PacketError, RawPacket, ServerPacket,
};

use std::io::Read;
use thiserror::Error;

pub mod authentication;
mod client_packet;
mod container;
pub mod player_packet;

#[derive(Clone)]
pub struct PlayerConfig {
    pub locale: String, // 16
    pub view_distance: i8,
    pub chat_mode: ChatMode,
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: Hand,
    pub text_filtering: bool,
    pub server_listing: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            locale: "en_us".to_string(),
            view_distance: 2,
            chat_mode: ChatMode::Enabled,
            chat_colors: true,
            skin_parts: 0,
            main_hand: Hand::Main,
            text_filtering: false,
            server_listing: false,
        }
    }
}

pub struct Client {
    pub gameprofile: Mutex<Option<GameProfile>>,

    pub config: Mutex<Option<PlayerConfig>>,
    pub brand: Mutex<Option<String>>,

    pub protocol_version: AtomicI32,
    pub connection_state: AtomicCell<ConnectionState>,
    pub encryption: AtomicBool,
    pub closed: AtomicBool,
    pub token: Token,
    pub connection: Arc<Mutex<TcpStream>>,
    pub address: Mutex<SocketAddr>,
    enc: Arc<Mutex<PacketEncoder>>,
    dec: Arc<Mutex<PacketDecoder>>,
    pub client_packets_queue: Arc<Mutex<Vec<RawPacket>>>,

    pub make_player: AtomicBool,
}

impl Client {
    pub fn new(token: Token, connection: TcpStream, address: SocketAddr) -> Self {
        Self {
            protocol_version: AtomicI32::new(0),
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            brand: Mutex::new(None),
            token,
            address: Mutex::new(address),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            connection: Arc::new(Mutex::new(connection)),
            enc: Arc::new(Mutex::new(PacketEncoder::default())),
            dec: Arc::new(Mutex::new(PacketDecoder::default())),
            encryption: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            client_packets_queue: Arc::new(Mutex::new(Vec::new())),
            make_player: AtomicBool::new(false),
        }
    }

    /// adds a Incoming packet to the queue
    pub fn add_packet(&self, packet: RawPacket) {
        let mut client_packets_queue = self.client_packets_queue.lock();
        client_packets_queue.push(packet);
    }

    /// enables encryption
    pub fn enable_encryption(
        &self,
        shared_secret: &[u8], // decrypted
    ) -> Result<(), EncryptionError> {
        self.encryption
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let crypt_key: [u8; 16] = shared_secret
            .try_into()
            .map_err(|_| EncryptionError::SharedWrongLength)?;
        self.dec.lock().enable_encryption(&crypt_key);
        self.enc.lock().enable_encryption(&crypt_key);
        Ok(())
    }

    // Compression threshold, Compression level
    pub fn set_compression(&self, compression: Option<(u32, u32)>) {
        self.dec.lock().set_compression(compression.map(|v| v.0));
        self.enc.lock().set_compression(compression);
    }

    /// Send a Clientbound Packet to the Client
    pub fn send_packet<P: ClientPacket>(&self, packet: &P) {
        // assert!(!self.closed);
        let mut enc = self.enc.lock();
        enc.append_packet(packet)
            .unwrap_or_else(|e| self.kick(&e.to_string()));
        self.connection
            .lock()
            .write_all(&enc.take())
            .map_err(|_| PacketError::ConnectionWrite)
            .unwrap_or_else(|e| self.kick(&e.to_string()));
    }

    pub fn try_send_packet<P: ClientPacket>(&self, packet: &P) -> Result<(), PacketError> {
        // assert!(!self.closed);

        let mut enc = self.enc.lock();
        enc.append_packet(packet)?;
        self.connection
            .lock()
            .write_all(&enc.take())
            .map_err(|_| PacketError::ConnectionWrite)?;
        Ok(())
    }

    pub async fn process_packets(&self, server: &Arc<Server>) {
        while let Some(mut packet) = self.client_packets_queue.lock().pop() {
            match self.handle_packet(server, &mut packet).await {
                Ok(_) => {}
                Err(e) => {
                    let text = format!("Error while reading incoming packet {}", e);
                    log::error!("{}", text);
                    self.kick(&text)
                }
            };
        }
    }

    /// Handles an incoming decoded not Play state Packet
    pub async fn handle_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        // TODO: handle each packet's Error instead of calling .unwrap()
        let bytebuf = &mut packet.bytebuf;
        match self.connection_state.load() {
            pumpkin_protocol::ConnectionState::HandShake => match packet.id.0 {
                SHandShake::PACKET_ID => {
                    self.handle_handshake(server, SHandShake::read(bytebuf)?);
                    Ok(())
                }
                _ => {
                    log::error!(
                        "Failed to handle packet id {} while in Handshake state",
                        packet.id.0
                    );
                    Ok(())
                }
            },
            pumpkin_protocol::ConnectionState::Status => match packet.id.0 {
                SStatusRequest::PACKET_ID => {
                    self.handle_status_request(server, SStatusRequest::read(bytebuf)?);
                    Ok(())
                }
                SStatusPingRequest::PACKET_ID => {
                    self.handle_ping_request(server, SStatusPingRequest::read(bytebuf)?);
                    Ok(())
                }
                _ => {
                    log::error!(
                        "Failed to handle packet id {} while in Status state",
                        packet.id.0
                    );
                    Ok(())
                }
            },
            // TODO: Check config if transfer is enabled
            pumpkin_protocol::ConnectionState::Login
            | pumpkin_protocol::ConnectionState::Transfer => match packet.id.0 {
                SLoginStart::PACKET_ID => {
                    self.handle_login_start(server, SLoginStart::read(bytebuf)?);
                    Ok(())
                }
                SEncryptionResponse::PACKET_ID => {
                    self.handle_encryption_response(server, SEncryptionResponse::read(bytebuf)?)
                        .await;
                    Ok(())
                }
                SLoginPluginResponse::PACKET_ID => {
                    self.handle_plugin_response(server, SLoginPluginResponse::read(bytebuf)?);
                    Ok(())
                }
                SLoginAcknowledged::PACKET_ID => {
                    self.handle_login_acknowledged(server, SLoginAcknowledged::read(bytebuf)?);
                    Ok(())
                }
                _ => {
                    log::error!(
                        "Failed to handle packet id {} while in Login state",
                        packet.id.0
                    );
                    Ok(())
                }
            },
            pumpkin_protocol::ConnectionState::Config => match packet.id.0 {
                SClientInformationConfig::PACKET_ID => {
                    self.handle_client_information_config(
                        server,
                        SClientInformationConfig::read(bytebuf)?,
                    );
                    Ok(())
                }
                SPluginMessage::PACKET_ID => {
                    self.handle_plugin_message(server, SPluginMessage::read(bytebuf)?);
                    Ok(())
                }
                SAcknowledgeFinishConfig::PACKET_ID => {
                    self.handle_config_acknowledged(
                        server,
                        SAcknowledgeFinishConfig::read(bytebuf)?,
                    )
                    .await;
                    Ok(())
                }
                SKnownPacks::PACKET_ID => {
                    self.handle_known_packs(server, SKnownPacks::read(bytebuf)?);
                    Ok(())
                }
                _ => {
                    log::error!(
                        "Failed to handle packet id {} while in Config state",
                        packet.id.0
                    );
                    Ok(())
                }
            },
            _ => {
                log::error!("Invalid Connection state {:?}", self.connection_state);
                Ok(())
            }
        }
    }

    /// Reads the connection until our buffer of len 4096 is full, then decode
    /// Close connection when an error occurs or when the Client closed the connection
    pub async fn poll(&self, event: &Event) {
        if event.is_readable() {
            let mut received_data = vec![0; 4096];
            let mut bytes_read = 0;
            loop {
                let connection = self.connection.clone();
                let mut connection = connection.lock();
                match connection.read(&mut received_data[bytes_read..]) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        self.close();
                        break;
                    }
                    Ok(n) => {
                        bytes_read += n;
                        received_data.extend(&vec![0; n]);
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
                let mut dec = self.dec.lock();
                dec.queue_slice(&received_data[..bytes_read]);
                match dec.decode() {
                    Ok(packet) => {
                        if let Some(packet) = packet {
                            self.add_packet(packet);
                        }
                    }
                    Err(err) => self.kick(&err.to_string()),
                }
                dec.clear();
            }
        }
    }

    /// Kicks the Client with a reason depending on the connection state
    pub fn kick(&self, reason: &str) {
        dbg!(reason);
        match self.connection_state.load() {
            ConnectionState::Login => {
                self.try_send_packet(&CLoginDisconnect::new(
                    &serde_json::to_string_pretty(&reason).unwrap_or("".into()),
                ))
                .unwrap_or_else(|_| self.close());
            }
            ConnectionState::Config => {
                self.try_send_packet(&CConfigDisconnect::new(reason))
                    .unwrap_or_else(|_| self.close());
            }
            // So we can also kick on errors, but generally should use Player::kick
            ConnectionState::Play => {
                self.try_send_packet(&CPlayDisconnect::new(&TextComponent::text(reason)))
                    .unwrap_or_else(|_| self.close());
            }
            _ => {
                log::warn!("Can't kick in {:?} State", self.connection_state)
            }
        }
        self.close()
    }

    /// You should prefer to use `kick` when you can
    pub fn close(&self) {
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
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
