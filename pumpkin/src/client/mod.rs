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
    bytebuf::{packet_id::Packet, ReadingError},
    client::{config::CConfigDisconnect, login::CLoginDisconnect, play::CPlayDisconnect},
    packet_decoder::PacketDecoder,
    packet_encoder::{PacketEncodeError, PacketEncoder},
    server::{
        config::{SAcknowledgeFinishConfig, SClientInformationConfig, SKnownPacks, SPluginMessage},
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
        status::{SStatusPingRequest, SStatusRequest},
    },
    ClientPacket, ConnectionState, RawPacket, ServerPacket,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use pumpkin_protocol::server::config::SCookieResponse as SCCookieResponse;
use pumpkin_protocol::server::login::SCookieResponse as SLCookieResponse;
use thiserror::Error;
pub mod authentication;
mod client_packet;
pub mod combat;
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
    pub view_distance: u8,
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
            main_hand: Hand::Right,
            text_filtering: false,
            server_listing: false,
        }
    }
}

/// Everything which makes a Connection with our Server is a `Client`.
/// Client will become Players when they reach the `Play` state
pub struct Client {
    /// The client id. This is good for coorelating a connection with a player
    /// Only used for logging purposes
    pub id: u16,
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
    pub fn new(connection: tokio::net::TcpStream, address: SocketAddr, id: u16) -> Self {
        let (connection_reader, connection_writer) = connection.into_split();
        Self {
            id,
            protocol_version: AtomicI32::new(0),
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            brand: Mutex::new(None),
            server_address: Mutex::new(String::new()),
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

    /// Enables or disables packet encryption for the connection.
    ///
    /// This function takes an optional shared secret as input. If the shared secret is provided,
    /// the connection's encryption is enabled using the provided secret key. Otherwise, encryption is disabled.
    ///
    /// # Arguments
    ///
    /// * `shared_secret`: An optional **already decrypted** shared secret key used for encryption.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the encryption was set successfully.
    ///
    /// # Errors
    ///
    /// Returns an `EncryptionError` if the shared secret has an incorrect length.
    ///
    /// # Examples
    /// ```
    ///  let shared_secret = server.decrypt(&encryption_response.shared_secret).unwrap();
    ///
    ///  if let Err(error) = self.set_encryption(Some(&shared_secret)).await {
    ///       self.kick(&error.to_string()).await;
    ///       return;
    ///  }
    /// ```
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

    /// Enables or disables packet compression for the connection.
    ///
    /// This function takes an optional `CompressionInfo` struct as input. If the `CompressionInfo` is provided,
    /// packet compression is enabled with the specified threshold. Otherwise, compression is disabled.
    ///
    /// # Arguments
    ///
    /// * `compression`: An optional `CompressionInfo` struct containing the compression threshold and compression level.
    pub async fn set_compression(&self, compression: Option<CompressionInfo>) {
        self.dec.lock().await.set_compression(compression.is_some());
        self.enc.lock().await.set_compression(compression);
    }

    /// Sends a clientbound packet to the connected client.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    pub async fn send_packet<P: ClientPacket>(&self, packet: &P) {
        //log::debug!("Sending packet with id {} to {}", P::PACKET_ID, self.id);
        // assert!(!self.closed);
        if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let mut enc = self.enc.lock().await;
        if let Err(error) = enc.append_packet(packet) {
            self.kick(&error.to_string()).await;
            return;
        }

        let mut writer = self.connection_writer.lock().await;
        if let Err(error) = writer.write_all(&enc.take()).await {
            log::debug!("Unable to write to connection: {}", error.to_string());
        }

        /*
        else if let Err(error) = writer.flush().await {
            log::warn!(
                "Failed to flush writer for id {}: {}",
                self.id,
                error.to_string()
            );
        }
        */
    }

    /// Sends a clientbound packet to the connected client.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the Packet was Send successfully.
    ///
    /// # Errors
    ///
    /// Returns an `PacketError` if the could not be Send.
    pub async fn try_send_packet<P: ClientPacket>(
        &self,
        packet: &P,
    ) -> Result<(), PacketEncodeError> {
        // assert!(!self.closed);
        /*
        log::debug!(
            "Trying to send packet with id {} to {}",
            P::PACKET_ID,
            self.id
        );
        */

        let mut enc = self.enc.lock().await;
        enc.append_packet(packet)?;

        let mut writer = self.connection_writer.lock().await;
        let _ = writer.write_all(&enc.take()).await;

        /*
        writer
            .flush()
            .await
            .map_err(|_| PacketError::ConnectionWrite)?;
        */
        Ok(())
    }

    /// Processes all packets received from the connected client in a loop.
    ///
    /// This function continuously dequeues packets from the client's packet queue and processes them.
    /// Processing involves calling the `handle_packet` function with the server instance and the packet itself.
    ///
    /// The loop exits when:
    ///
    /// - The connection is closed (checked before processing each packet).
    /// - An error occurs while processing a packet (client is kicked with an error message).
    ///
    /// # Arguments
    ///
    /// * `server`: A reference to the `Arc<Server>` instance.
    pub async fn process_packets(&self, server: &Arc<Server>) {
        let mut packet_queue = self.client_packets_queue.lock().await;
        while let Some(mut packet) = packet_queue.pop_front() {
            if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
                log::debug!("Canceling client packet processing (pre)");
                return;
            }
            if let Err(error) = self.handle_packet(server, &mut packet).await {
                let text = format!("Error while reading incoming packet {error}");
                log::error!(
                    "Failed to read incoming packet with id {}: {}",
                    i32::from(packet.id),
                    error
                );
                self.kick(&text).await;
            };
        }
    }

    /// Handles an incoming packet, routing it to the appropriate handler based on the current connection state.
    ///
    /// This function takes a `RawPacket` and routes it to the corresponding handler based on the current connection state.
    /// It supports the following connection states:
    ///
    /// - **Handshake:** Handles handshake packets.
    /// - **Status:** Handles status request and ping packets.
    /// - **Login/Transfer:** Handles login and transfer packets.
    /// - **Config:** Handles configuration packets.
    ///
    /// For the `Play` state, an error is logged as it indicates an invalid state for packet processing.
    ///
    /// # Arguments
    ///
    /// * `server`: A reference to the `Arc<Server>` instance.
    /// * `packet`: A mutable reference to the `RawPacket` to be processed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the packet was read and handled successfully.
    ///
    /// # Errors
    ///
    /// Returns a `DeserializerError` if an error occurs during packet deserialization.
    pub async fn handle_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), ReadingError> {
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

    async fn handle_handshake_packet(&self, packet: &mut RawPacket) -> Result<(), ReadingError> {
        log::debug!("Handling handshake group");
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            0 => {
                self.handle_handshake(SHandShake::read(bytebuf)?).await;
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} in Handshake state",
                    packet.id.0
                );
            }
        };
        Ok(())
    }

    async fn handle_status_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling status group");
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SStatusRequest::PACKET_ID => {
                self.handle_status_request(server).await;
            }
            SStatusPingRequest::PACKET_ID => {
                self.handle_ping_request(SStatusPingRequest::read(bytebuf)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle client packet id {} in Status State",
                    packet.id.0
                );
            }
        };

        Ok(())
    }

    async fn handle_login_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling login group for id");
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
                self.handle_login_acknowledged(server).await;
            }
            SLCookieResponse::PACKET_ID => {
                self.handle_login_cookie_response(SLCookieResponse::read(bytebuf)?);
            }
            _ => {
                log::error!(
                    "Failed to handle client packet id {} in Login State",
                    packet.id.0
                );
            }
        };
        Ok(())
    }

    async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling config group");
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
                self.handle_config_acknowledged();
            }
            SKnownPacks::PACKET_ID => {
                self.handle_known_packs(server, SKnownPacks::read(bytebuf)?)
                    .await;
            }
            SCCookieResponse::PACKET_ID => {
                self.handle_config_cookie_response(SCCookieResponse::read(bytebuf)?);
            }
            _ => {
                log::error!(
                    "Failed to handle client packet id {} in Config State",
                    packet.id.0
                );
            }
        };
        Ok(())
    }

    /// Reads the connection until our buffer of len 4096 is full, then decode
    /// Close connection when an error occurs or when the Client closed the connection
    /// Returns if connection is still open
    pub async fn poll(&self) -> bool {
        loop {
            if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
                // If we manually close (like a kick) we dont want to keep reading bytes
                return false;
            }

            let mut dec = self.dec.lock().await;

            match dec.decode() {
                Ok(Some(packet)) => {
                    self.add_packet(packet).await;
                    return true;
                }
                Ok(None) => (), //log::debug!("Waiting for more data to complete packet..."),
                Err(err) => {
                    log::warn!("Failed to decode packet for: {}", err.to_string());
                    self.close();
                    return false; // return to avoid reserving additional bytes
                }
            }

            dec.reserve(4096);
            let mut buf = dec.take_capacity();

            let bytes_read = self.connection_reader.lock().await.read_buf(&mut buf).await;
            match bytes_read {
                Ok(cnt) => {
                    //log::debug!("Read {} bytes", cnt);
                    if cnt == 0 {
                        self.close();
                        return false;
                    }
                }
                Err(error) => {
                    log::error!("Error while reading incoming packet {}", error);
                    self.close();
                    return false;
                }
            };

            // This should always be an O(1) unsplit because we reserved space earlier and
            // the call to `read_buf` shouldn't have grown the allocation.
            dec.queue_bytes(buf);
        }
    }

    /// Disconnects a client from the server with a specified reason.
    ///
    /// This function kicks a client identified by its ID from the server. The appropriate disconnect packet is sent based on the client's current connection state.
    ///
    /// # Arguments
    ///
    /// * `reason`: A string describing the reason for kicking the client.
    pub async fn kick(&self, reason: &str) {
        log::info!("Kicking Client id {} for {}", self.id, reason);
        let result = match self.connection_state.load() {
            ConnectionState::Login => {
                self.try_send_packet(&CLoginDisconnect::new(
                    &serde_json::to_string_pretty(&reason).unwrap_or_else(|_| String::new()),
                ))
                .await
            }
            ConnectionState::Config => self.try_send_packet(&CConfigDisconnect::new(reason)).await,
            // This way players get kicked when players using client functions (e.g. poll, send_packet)
            ConnectionState::Play => {
                self.try_send_packet(&CPlayDisconnect::new(&TextComponent::text(reason)))
                    .await
            }
            _ => {
                log::warn!("Can't kick in {:?} State", self.connection_state);
                Ok(())
            }
        };
        if let Err(err) = result {
            log::warn!("Failed to kick {}: {}", self.id, err.to_string());
        }
        log::debug!("Closing connection for {}", self.id);
        self.close();
    }

    /// Closes the connection to the client.
    ///
    /// This function marks the connection as closed using an atomic flag. It's generally preferable
    /// to use the `kick` function if you want to send a specific message to the client explaining the reason for the closure.
    /// However, use `close` in scenarios where sending a message is not critical or might not be possible (e.g., sudden connection drop).
    ///
    /// # Notes
    ///
    /// This function does not attempt to send any disconnect packets to the client.
    pub fn close(&self) {
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        log::debug!("Closed connection for {}", self.id);
    }
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("failed to decrypt shared secret")]
    FailedDecrypt,
    #[error("shared secret has the wrong length")]
    SharedWrongLength,
}
