use std::{
    collections::{HashMap, VecDeque},
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
use num_traits::FromPrimitive;
use pumpkin_config::compression::CompressionInfo;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::{
    bytebuf::DeserializerError,
    client::{config::CConfigDisconnect, login::CLoginDisconnect, play::CPlayDisconnect},
    packet_decoder::PacketDecoder,
    packet_encoder::PacketEncoder,
    server::{
        config::{
            SAcknowledgeFinishConfig, SClientInformationConfig, SKnownPacks, SPluginMessage,
            ServerboundConfigPackets,
        },
        handshake::SHandShake,
        login::{
            SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart,
            ServerboundLoginPackets,
        },
        status::{SStatusPingRequest, SStatusRequest, ServerboundStatusPackets},
    },
    ClientPacket, ConnectionState, PacketError, RawPacket, ServerPacket,
};
use tokio::sync::{Mutex, Notify};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    task::JoinHandle,
};

use thiserror::Error;

pub mod authentication;
mod client_packet;
mod combat;
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

pub struct HandleWrapper {
    handle: Option<JoinHandle<()>>,
    aborted: bool,
}

impl HandleWrapper {
    #[must_use]
    pub fn new(handle: JoinHandle<()>) -> Self {
        Self {
            handle: Some(handle),
            aborted: false,
        }
    }

    pub fn abort(&mut self) {
        self.aborted = true;
        if let Some(handle) = &self.handle {
            handle.abort();
        } else {
            log::error!("Trying to abort without a handle!");
        }
    }

    pub fn take_handle(&mut self) -> JoinHandle<()> {
        self.handle.take().unwrap()
    }

    #[must_use]
    pub fn aborted(&self) -> bool {
        self.aborted
    }
}

pub type TaskId = usize;

pub struct ExpensiveTaskWatcher {
    tasks: HashMap<TaskId, HandleWrapper>,
    // usize is the greatest we can realistically index anything by so this should be fine
    unique_index: TaskId,
}

impl ExpensiveTaskWatcher {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            unique_index: 0,
        }
    }

    /// Returns the task id of the handle being watched
    pub fn add_handle(&mut self, handle: JoinHandle<()>) -> TaskId {
        let new_index = self.unique_index;

        // Ensure our next key isn't already in use
        for added in 1..usize::MAX {
            let next_index = self.unique_index.wrapping_add(added);

            if next_index == new_index {
                break;
            }

            if !self.tasks.contains_key(&next_index) {
                self.unique_index = next_index;
                break;
            }
        }

        if self.unique_index == new_index {
            // Not a death sentence, just cant await / cancel some tasks
            log::error!("Used all indices in the expensive task watcher!");
        }

        let wrapped_handle = HandleWrapper::new(handle);
        self.tasks.insert(new_index, wrapped_handle);
        new_index
    }

    /// If the id exists, remove the task without canceling it and return the wrapper if it exists
    pub fn remove_task(&mut self, id: &TaskId) -> Option<HandleWrapper> {
        if let Some(wrapper) = self.tasks.remove(id) {
            return Some(wrapper);
        }
        None
    }

    /// If the id exists, return if the task was aborted, otherwise return false
    #[must_use]
    pub fn is_aborted(&self, id: &TaskId) -> bool {
        if let Some(wrapper) = self.tasks.get(id) {
            return wrapper.aborted();
        }
        false
    }

    /// If the id exists, cancel the task and return if the operation was successful
    pub fn cancel_task(&mut self, id: &TaskId, reason: &str) -> bool {
        if let Some(wrapper) = self.tasks.get_mut(id) {
            if !wrapper.aborted() {
                log::debug!("Canceling task {}: {}", id, reason);
                wrapper.abort();
                return true;
            }
        }
        false
    }

    /// Cancel all tasks currently being watched
    pub fn cancel_all_tasks(&mut self, reason: &str) {
        self.tasks.iter_mut().for_each(|(id, task)| {
            if !task.aborted() {
                log::debug!("Canceling task {}: {}", id, reason);
                task.abort();
            }
        });
    }

    /// Return all handles currently being watched. ``Self::cancel_all_tasks`` should not be called after
    /// this
    pub fn dump_all_tasks(&mut self) -> Vec<JoinHandle<()>> {
        self.tasks
            .values_mut()
            .map(HandleWrapper::take_handle)
            .collect()
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

    /// Tell tasks to stop if we are closing
    cancel_tasks: Notify,
    /// Tasks that make sense to cancel if a connection closes/will close
    expensive_tasks: parking_lot::RwLock<ExpensiveTaskWatcher>,
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
            cancel_tasks: Notify::new(),
            expensive_tasks: parking_lot::RwLock::new(ExpensiveTaskWatcher::new()),
        }
    }

    pub fn watch_expensive_tasks(&self, tasks: Vec<JoinHandle<()>>) -> Vec<TaskId> {
        let mut expensive_tasks = self.expensive_tasks.write();
        tasks
            .into_iter()
            .map(|task| expensive_tasks.add_handle(task))
            .collect()
    }

    pub fn cancel_expensive_task(&self, id: &TaskId, reason: &str) -> bool {
        let mut expensive_tasks = self.expensive_tasks.write();
        expensive_tasks.cancel_task(id, reason)
    }

    pub fn is_expensive_task_cancelled(&self, id: &TaskId) -> bool {
        let expensive_tasks = self.expensive_tasks.read();
        expensive_tasks.is_aborted(id)
    }

    pub fn watch_expensive_task(&self, task: JoinHandle<()>) -> TaskId {
        let mut expensive_tasks = self.expensive_tasks.write();
        expensive_tasks.add_handle(task)
    }

    pub fn stop_watching_expensive_task(&self, id: &TaskId) -> Option<HandleWrapper> {
        let mut expensive_tasks = self.expensive_tasks.write();
        expensive_tasks.remove_task(id)
    }

    pub fn cancel_expensive_tasks(&self, reason: &str) {
        let mut expensive_tasks = self.expensive_tasks.write();
        expensive_tasks.cancel_all_tasks(reason);
    }

    pub async fn await_expensive_tasks(&self, reason: &str) {
        let handles = {
            let mut expensive_tasks = self.expensive_tasks.write();
            expensive_tasks.dump_all_tasks()
        };

        log::debug!("Awaiting all watched tasks: {}", reason);
        for handle in handles {
            let _ = handle.await;
        }
        log::debug!("Done awaiting all watched tasks: {}", reason);
    }

    pub async fn await_cancel_notify(&self) {
        self.cancel_tasks.notified().await;
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
        //log::debug!("Sending packet with id {} to {}", P::PACKET_ID, self.id);
        // assert!(!self.closed);
        let mut enc = self.enc.lock().await;
        if let Err(error) = enc.append_packet(packet) {
            if error.kickable() {
                self.kick(&error.to_string()).await;
            }
            return;
        }

        let mut writer = self.connection_writer.lock().await;
        if let Err(error) = writer
            .write_all(&enc.take())
            .await
            .map_err(|_| PacketError::ConnectionWrite)
        {
            if error.kickable() {
                self.kick(&error.to_string()).await;
            }
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

    pub async fn try_send_packet<P: ClientPacket>(&self, packet: &P) -> Result<(), PacketError> {
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
        writer
            .write_all(&enc.take())
            .await
            .map_err(|_| PacketError::ConnectionWrite)?;

        /*
        writer
            .flush()
            .await
            .map_err(|_| PacketError::ConnectionWrite)?;
        */
        Ok(())
    }

    /// Processes all packets send by the client
    pub async fn process_packets(&self, server: &Arc<Server>) {
        let mut packet_queue = self.client_packets_queue.lock().await;
        while let Some(mut packet) = packet_queue.pop_front() {
            if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
                log::debug!("Canceling client packet processing (pre)");
                return;
            }

            tokio::select! {
                () = self.cancel_tasks.notified() => {
                    log::debug!("Canceling client packet processing (interrupt)");
                    return;
                },
                packet_result = self.handle_packet(server, &mut packet) => {
                    if let Err(error) = packet_result {
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
    ) -> Result<(), DeserializerError> {
        log::debug!("Handling status group");
        let bytebuf = &mut packet.bytebuf;
        if let Some(packet) = ServerboundStatusPackets::from_i32(packet.id.0) {
            match packet {
                ServerboundStatusPackets::StatusRequest => {
                    self.handle_status_request(server, SStatusRequest::read(bytebuf)?)
                        .await;
                }
                ServerboundStatusPackets::PingRequest => {
                    self.handle_ping_request(SStatusPingRequest::read(bytebuf)?)
                        .await;
                }
            };
        } else {
            log::error!(
                "Failed to handle client packet id {:#04x} in Status State",
                packet.id.0
            );
            return Err(DeserializerError::UnknownPacket);
        };
        Ok(())
    }

    async fn handle_login_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        log::debug!("Handling login group for id");
        let bytebuf = &mut packet.bytebuf;
        if let Some(packet) = ServerboundLoginPackets::from_i32(packet.id.0) {
            match packet {
                ServerboundLoginPackets::LoginStart => {
                    self.handle_login_start(server, SLoginStart::read(bytebuf)?)
                        .await;
                }
                ServerboundLoginPackets::EncryptionResponse => {
                    self.handle_encryption_response(server, SEncryptionResponse::read(bytebuf)?)
                        .await;
                }
                ServerboundLoginPackets::PluginResponse => {
                    self.handle_plugin_response(SLoginPluginResponse::read(bytebuf)?)
                        .await;
                }
                ServerboundLoginPackets::LoginAcknowledged => {
                    self.handle_login_acknowledged(server, SLoginAcknowledged::read(bytebuf)?)
                        .await;
                }
                ServerboundLoginPackets::CookieResponse => {}
            };
        } else {
            log::error!(
                "Failed to handle client packet id {:#04x} in Login State",
                packet.id.0
            );
            return Ok(());
        };
        Ok(())
    }

    async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), DeserializerError> {
        log::debug!("Handling config group");
        let bytebuf = &mut packet.bytebuf;
        if let Some(packet) = ServerboundConfigPackets::from_i32(packet.id.0) {
            #[expect(clippy::match_same_arms)]
            match packet {
                ServerboundConfigPackets::ClientInformation => {
                    self.handle_client_information_config(SClientInformationConfig::read(bytebuf)?)
                        .await;
                }
                ServerboundConfigPackets::CookieResponse => {}
                ServerboundConfigPackets::PluginMessage => {
                    self.handle_plugin_message(SPluginMessage::read(bytebuf)?)
                        .await;
                }
                ServerboundConfigPackets::AcknowledgedFinish => {
                    self.handle_config_acknowledged(&SAcknowledgeFinishConfig::read(bytebuf)?);
                }
                ServerboundConfigPackets::KeepAlive => {}
                ServerboundConfigPackets::Pong => {}
                ServerboundConfigPackets::ResourcePackResponse => {}
                ServerboundConfigPackets::KnownPacks => {
                    self.handle_known_packs(server, SKnownPacks::read(bytebuf)?)
                        .await;
                }
            };
        } else {
            log::error!(
                "Failed to handle client packet id {:#04x} in Config State",
                packet.id.0
            );
            return Err(DeserializerError::UnknownPacket);
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
                Err(err) => log::warn!("Failed to decode packet for: {}", err.to_string()),
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

    /// Kicks the Client with a reason depending on the connection state
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

    /// You should prefer to use `kick` when you can
    pub fn close(&self) {
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // This only truely stops handling packets after we mark closed to be true
        self.cancel_tasks.notify_waiters();
        // Cancel tasks ASAP, Clean up any work we picked up between the caller and now (if not
        // called directly)
        self.cancel_expensive_tasks("Closing client");

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
