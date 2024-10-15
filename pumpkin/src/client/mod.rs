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
use mio::{event::Event, net::TcpStream};
use parking_lot::Mutex;
use pumpkin_config::compression::CompressionInfo;
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

/// Represents a player's configuration settings.
///
/// This struct contains various options that can be customized by the player, affecting their gameplay experience.
///
/// **Usage:**
///
/// This struct is typically used to store and manage a player's preferences. It can be sent to the server when a player joins or when they change their settings.
#[derive(Clone)]
pub struct PlayerConfig {
    /// The player's preferred language.
    pub locale: String, // 16
    /// The maximum distance at which chunks are rendered.
    pub view_distance: i8,
    /// The player's chat mode settings
    pub chat_mode: ChatMode,
    /// Whether chat colors are enabled.
    pub chat_colors: bool,
    /// The player's skin configuration options.
    pub skin_parts: u8,
    /// The player's dominant hand (left or right).
    pub main_hand: Hand,
    /// Whether text filtering is enabled.
    pub text_filtering: bool,
    /// Whether the player wants to appear in the server list.
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

/// Everything which makes a Connection with our Server is a `Client`.
/// Client will become Players when they reach the `Play` state
pub struct Client {
    /// The client's game profile information.
    pub gameprofile: Mutex<Option<GameProfile>>,
    /// The client's configuration settings, Optional
    pub config: Mutex<Option<PlayerConfig>>,
    /// The client's brand or modpack information, Optional.
    pub brand: Mutex<Option<String>>,
    /// The minecraft protocol version used by the client.
    pub protocol_version: AtomicI32,
    /// The current connection state of the client (e.g., Handshaking, Status, Play).
    pub connection_state: AtomicCell<ConnectionState>,
    /// Whether encryption is enabled for the connection.
    pub encryption: AtomicBool,
    /// Indicates if the client connection is closed.
    pub closed: AtomicBool,
    /// A unique id identifying the client.
    pub id: usize,
    /// The underlying TCP connection to the client.
    pub connection: Arc<Mutex<TcpStream>>,
    /// The client's IP address.
    pub address: Mutex<SocketAddr>,
    /// The packet encoder for outgoing packets.
    enc: Arc<Mutex<PacketEncoder>>,
    /// The packet decoder for incoming packets.
    dec: Arc<Mutex<PacketDecoder>>,
    /// A queue of raw packets received from the client, waiting to be processed.
    pub client_packets_queue: Arc<Mutex<Vec<RawPacket>>>,

    /// Indicates whether the client should be converted into a player.
    pub make_player: AtomicBool,
    /// Sends each keep alive packet that the server receives for a player to here, which gets picked up in a tokio task
    pub keep_alive_sender: Arc<tokio::sync::mpsc::Sender<i64>>,
    /// Stores the last time it was confirmed that the client is alive
    pub last_alive_received: AtomicCell<std::time::Instant>,
}

impl Client {
    pub fn new(
        id: usize,
        connection: TcpStream,
        address: SocketAddr,
        keep_alive_sender: Arc<tokio::sync::mpsc::Sender<i64>>,
    ) -> Self {
        Self {
            protocol_version: AtomicI32::new(0),
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            brand: Mutex::new(None),
            id,
            address: Mutex::new(address),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            connection: Arc::new(Mutex::new(connection)),
            enc: Arc::new(Mutex::new(PacketEncoder::default())),
            dec: Arc::new(Mutex::new(PacketDecoder::default())),
            encryption: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            client_packets_queue: Arc::new(Mutex::new(Vec::new())),
            make_player: AtomicBool::new(false),
            keep_alive_sender,
            last_alive_received: AtomicCell::new(std::time::Instant::now()),
        }
    }

    /// Adds a Incoming packet to the queue
    pub fn add_packet(&self, packet: RawPacket) {
        let mut client_packets_queue = self.client_packets_queue.lock();
        client_packets_queue.push(packet);
    }

    /// Sets the Packet encryption
    pub fn set_encryption(
        &self,
        shared_secret: Option<&[u8]>, // decrypted
    ) -> Result<(), EncryptionError> {
        if let Some(shared_secret) = shared_secret {
            self.encryption
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let crypt_key: [u8; 16] = shared_secret
                .try_into()
                .map_err(|_| EncryptionError::SharedWrongLength)?;
            self.dec.lock().set_encryption(Some(&crypt_key));
            self.enc.lock().set_encryption(Some(&crypt_key));
        } else {
            self.dec.lock().set_encryption(None);
            self.enc.lock().set_encryption(None);
        }
        Ok(())
    }

    /// Sets the Packet compression
    pub fn set_compression(&self, compression: Option<CompressionInfo>) {
        self.dec.lock().set_compression(compression.is_some());
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

    /// Processes all packets send by the client
    pub async fn process_packets(&self, server: &Arc<Server>) {
        while let Some(mut packet) = self.client_packets_queue.lock().pop() {
            let _ = self.handle_packet(server, &mut packet).await.map_err(|e| {
                let text = format!("Error while reading incoming packet {}", e);
                log::error!("{}", text);
                self.kick(&text)
            });
        }
    }

    /// Handles an incoming decoded not Play state Packet
    pub async fn handle_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        match self.connection_state.load() {
            pumpkin_protocol::ConnectionState::HandShake => self.handle_handshake_packet(packet),
            pumpkin_protocol::ConnectionState::Status => self.handle_status_packet(server, packet),
            // TODO: Check config if transfer is enabled
            pumpkin_protocol::ConnectionState::Login
            | pumpkin_protocol::ConnectionState::Transfer => {
                self.handle_login_packet(server, packet).await
            }
            pumpkin_protocol::ConnectionState::Config => {
                self.handle_config_packet(server, packet).await
            }
            _ => {
                log::error!("Invalid Connection state {:?}", self.connection_state);
                Ok(())
            }
        }
    }

    fn handle_handshake_packet(&self, packet: &mut RawPacket) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SHandShake::PACKET_ID => {
                self.handle_handshake(SHandShake::read(bytebuf)?);
                Ok(())
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} while in Handshake state",
                    packet.id.0
                );
                Ok(())
            }
        }
    }

    fn handle_status_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SStatusRequest::PACKET_ID => {
                self.handle_status_request(server, SStatusRequest::read(bytebuf)?);
                Ok(())
            }
            SStatusPingRequest::PACKET_ID => {
                self.handle_ping_request(SStatusPingRequest::read(bytebuf)?);
                Ok(())
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} while in Status state",
                    packet.id.0
                );
                Ok(())
            }
        }
    }

    async fn handle_login_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
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
                self.handle_plugin_response(SLoginPluginResponse::read(bytebuf)?);
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
        }
    }

    async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SClientInformationConfig::PACKET_ID => {
                self.handle_client_information_config(SClientInformationConfig::read(bytebuf)?);
                Ok(())
            }
            SPluginMessage::PACKET_ID => {
                self.handle_plugin_message(SPluginMessage::read(bytebuf)?);
                Ok(())
            }
            SAcknowledgeFinishConfig::PACKET_ID => {
                self.handle_config_acknowledged(SAcknowledgeFinishConfig::read(bytebuf)?)
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
        }
    }

    /// Reads the connection until our buffer of len 4096 is full, then decode
    /// Close connection when an error occurs or when the Client closed the connection
    pub async fn poll(&self, event: &Event) {
        if event.is_readable() {
            let mut received_data = vec![];
            let mut buf = [0; 4096];
            loop {
                let connection = self.connection.clone();
                let mut connection = connection.lock();
                match connection.read(&mut buf) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        self.close();
                        break;
                    }
                    Ok(n) => received_data.extend(&buf[..n]),
                    // Would block "errors" are the OS's way of saying that the
                    // connection is not actually ready to perform this I/O operation.
                    Err(ref err) if would_block(err) => break,
                    Err(ref err) if interrupted(err) => continue,
                    // Other errors we'll consider fatal.
                    Err(_) => self.close(),
                }
            }

            if !received_data.is_empty() {
                let mut dec = self.dec.lock();
                dec.queue_slice(&received_data);
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
                    &serde_json::to_string_pretty(&reason).unwrap_or_else(|_| "".into()),
                ))
                .unwrap_or_else(|_| self.close());
            }
            ConnectionState::Config => {
                self.try_send_packet(&CConfigDisconnect::new(reason))
                    .unwrap_or_else(|_| self.close());
            }
            // This way players get kicked when players using client functions (e.g. poll, send_packet)
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
