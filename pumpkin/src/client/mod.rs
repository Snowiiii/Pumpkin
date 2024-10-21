use std::{
    collections::VecDeque,
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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

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
    /// The Address used to connect to the Server, Send in the Handshake
    pub server_address: Mutex<String>,
    /// The current connection state of the client (e.g., Handshaking, Status, Play).
    pub connection_state: AtomicCell<ConnectionState>,
    /// Whether encryption is enabled for the connection.
    pub encryption: AtomicBool,
    /// Indicates if the client connection is closed.
    pub closed: AtomicBool,
    /// A unique id identifying the client.
    pub id: usize,
    /// The underlying TCP connection to the client.
    pub connection_reader: Arc<Mutex<tokio::net::tcp::OwnedReadHalf>>,
    pub connection_writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
    /// The client's IP address.
    pub address: Mutex<SocketAddr>,
    /// The packet encoder for outgoing packets.
    enc: Arc<Mutex<PacketEncoder>>,
    /// The packet decoder for incoming packets.
    dec: Arc<Mutex<PacketDecoder>>,
    /// A queue of raw packets received from the client, waiting to be processed.
    pub client_packets_queue: Arc<Mutex<VecDeque<RawPacket>>>,
    /// Indicates whether the client should be converted into a player.
    pub make_player: AtomicBool,
}

impl Client {
    #[must_use]
    pub fn new(id: usize, connection: tokio::net::TcpStream, address: SocketAddr) -> Self {
        let (connection_reader, connection_writer) = connection.into_split();
        Self {
            protocol_version: AtomicI32::new(0),
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            brand: Mutex::new(None),
            server_address: Mutex::new(String::new()),
            id,
            address: Mutex::new(address),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            connection_reader: Arc::new(Mutex::new(connection_reader)),
            connection_writer: Arc::new(Mutex::new(connection_writer)),
            enc: Arc::new(Mutex::new(PacketEncoder::default())),
            dec: Arc::new(Mutex::new(PacketDecoder::default())),
            encryption: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            client_packets_queue: Arc::new(Mutex::new(VecDeque::new())),
            make_player: AtomicBool::new(false),
        }
    }

    /// Adds a Incoming packet to the queue
    pub async fn add_packet(&self, packet: RawPacket) {
        let mut client_packets_queue = self.client_packets_queue.lock().await;
        client_packets_queue.push_back(packet);
    }

    /// Sets the Packet encryption
    pub async fn set_encryption(
        &self,
        shared_secret: Option<&[u8]>, // decrypted
    ) -> Result<(), EncryptionError> {
        if let Some(shared_secret) = shared_secret {
            self.encryption
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let crypt_key: [u8; 16] = shared_secret
                .try_into()
                .map_err(|_| EncryptionError::SharedWrongLength)?;
            self.dec.lock().await.set_encryption(Some(&crypt_key));
            self.enc.lock().await.set_encryption(Some(&crypt_key));
        } else {
            self.dec.lock().await.set_encryption(None);
            self.enc.lock().await.set_encryption(None);
        }
        Ok(())
    }

    /// Sets the Packet compression
    pub async fn set_compression(&self, compression: Option<CompressionInfo>) {
        self.dec.lock().await.set_compression(compression.is_some());
        self.enc.lock().await.set_compression(compression);
    }

    /// Send a Clientbound Packet to the Client
    pub async fn send_packet<P: ClientPacket>(&self, packet: &P) {
        // assert!(!self.closed);
        let mut enc = self.enc.lock().await;
        if let Err(error) = enc.append_packet(packet) {
            self.kick(&error.to_string()).await;
            return;
        }
        if let Err(error) = self
            .connection_writer
            .lock()
            .await
            .write_all(&enc.take())
            .await
            .map_err(|_| PacketError::ConnectionWrite)
        {
            self.kick(&error.to_string()).await;
        }
    }

    pub async fn try_send_packet<P: ClientPacket>(&self, packet: &P) -> Result<(), PacketError> {
        // assert!(!self.closed);

        let mut enc = self.enc.lock().await;
        enc.append_packet(packet)?;
        self.connection_writer
            .lock()
            .await
            .write_all(&enc.take())
            .await
            .map_err(|_| PacketError::ConnectionWrite)?;
        Ok(())
    }

    /// Processes all packets send by the client
    pub async fn process_packets(&self, server: &Arc<Server>) {
        while let Some(mut packet) = self.client_packets_queue.lock().await.pop_front() {
            if let Err(error) = self.handle_packet(server, &mut packet).await {
                log::error!("Error while reading incoming packet {error}");
                self.kick(&text).await;
            };
        }
    }

    /// Handles an incoming decoded not Play state Packet
    pub async fn handle_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        match self.connection_state.load() {
            pumpkin_protocol::ConnectionState::HandShake => {
                self.handle_handshake_packet(packet).await
            }
            pumpkin_protocol::ConnectionState::Status => {
                self.handle_status_packet(server, packet).await
            }
            // TODO: Check config if transfer is enabled
            pumpkin_protocol::ConnectionState::Login
            | pumpkin_protocol::ConnectionState::Transfer => {
                self.handle_login_packet(server, packet).await
            }
            pumpkin_protocol::ConnectionState::Config => {
                self.handle_config_packet(server, packet).await
            }
            pumpkin_protocol::ConnectionState::Play => {
                log::error!("Invalid Connection state {:?}", self.connection_state);
                Ok(())
            }
        }
    }

    async fn handle_handshake_packet(
        &self,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        if packet.id.0 == SHandShake::PACKET_ID {
            self.handle_handshake(SHandShake::read(bytebuf)?).await;
        } else {
            log::error!(
                "Failed to handle packet id {} while in Handshake state",
                packet.id.0
            );
        }
        Ok(())
    }

    async fn handle_status_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SStatusRequest::PACKET_ID => {
                self.handle_status_request(server, SStatusRequest::read(bytebuf)?)
                    .await;
            }
            SStatusPingRequest::PACKET_ID => {
                self.handle_ping_request(SStatusPingRequest::read(bytebuf)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} while in Status state",
                    packet.id.0
                );
            }
        }
        Ok(())
    }

    async fn handle_login_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SLoginStart::PACKET_ID => {
                self.handle_login_start(server, SLoginStart::read(bytebuf)?)
                    .await;
            }
            SEncryptionResponse::PACKET_ID => {
                self.handle_encryption_response(server, SEncryptionResponse::read(bytebuf)?)
                    .await;
            }
            SLoginPluginResponse::PACKET_ID => {
                self.handle_plugin_response(SLoginPluginResponse::read(bytebuf)?)
                    .await;
            }
            SLoginAcknowledged::PACKET_ID => {
                self.handle_login_acknowledged(server, SLoginAcknowledged::read(bytebuf)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} while in Login state",
                    packet.id.0
                );
            }
        }
        Ok(())
    }

    async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SClientInformationConfig::PACKET_ID => {
                self.handle_client_information_config(SClientInformationConfig::read(bytebuf)?)
                    .await;
            }
            SPluginMessage::PACKET_ID => {
                self.handle_plugin_message(SPluginMessage::read(bytebuf)?)
                    .await;
            }
            SAcknowledgeFinishConfig::PACKET_ID => {
                self.handle_config_acknowledged(&SAcknowledgeFinishConfig::read(bytebuf)?);
            }
            SKnownPacks::PACKET_ID => {
                self.handle_known_packs(server, SKnownPacks::read(bytebuf)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} while in Config state",
                    packet.id.0
                );
            }
        }
        Ok(())
    }

    /// Reads the connection until our buffer of len 4096 is full, then decode
    /// Close connection when an error occurs or when the Client closed the connection
    /// Returns if connection is still open
    pub async fn poll(&self) -> bool {
        loop {
            let mut dec = self.dec.lock().await;
            if let Ok(Some(packet)) = dec.decode() {
                self.add_packet(packet).await;
                return true;
            };

            dec.reserve(4096);
            let mut buf = dec.take_capacity();

            match self.connection_reader.lock().await.read_buf(&mut buf).await {
                Ok(0) => {
                    self.close();
                    return false;
                }
                Err(error) => {
                    log::error!("Error while reading incoming packet {}", error);
                    self.close();
                    return false;
                }
                _ => {}
            }

            // This should always be an O(1) unsplit because we reserved space earlier and
            // the call to `read_buf` shouldn't have grown the allocation.
            dec.queue_bytes(buf);
        }
    }

    /// Kicks the Client with a reason depending on the connection state
    pub async fn kick(&self, reason: &str) {
        log::debug!("Kicking client with reason: {}", reason);
        match self.connection_state.load() {
            ConnectionState::Login => {
                self.try_send_packet(&CLoginDisconnect::new(
                    &serde_json::to_string_pretty(&reason).unwrap_or_else(|_| String::new()),
                ))
                .await
                .unwrap_or_else(|_| self.close());
            }
            ConnectionState::Config => {
                self.try_send_packet(&CConfigDisconnect::new(reason))
                    .await
                    .unwrap_or_else(|_| self.close());
            }
            // This way players get kicked when players using client functions (e.g. poll, send_packet)
            ConnectionState::Play => {
                self.try_send_packet(&CPlayDisconnect::new(&TextComponent::text(reason)))
                    .await
                    .unwrap_or_else(|_| self.close());
            }
            _ => {
                log::warn!("Can't kick in {:?} State", self.connection_state);
            }
        }
        self.close();
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
