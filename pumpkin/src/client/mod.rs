use std::{
    collections::VecDeque,
    io::{self, Write},
    net::SocketAddr,
    rc::Rc,
};

use crate::{
    entity::player::{ChatMode, GameMode, Hand, Player},
    server::Server,
};

use authentication::GameProfile;
use mio::{event::Event, net::TcpStream, Token};
use num_traits::ToPrimitive;
use pumpkin_protocol::{
    bytebuf::packet_id::Packet,
    client::{
        config::CConfigDisconnect,
        login::CLoginDisconnect,
        play::{CGameEvent, CPlayDisconnect, CSyncPlayerPostion, CSystemChatMessge},
    },
    packet_decoder::PacketDecoder,
    packet_encoder::PacketEncoder,
    server::{
        config::{SAcknowledgeFinishConfig, SClientInformation, SKnownPacks, SPluginMessage},
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
        play::{
            SChatCommand, SConfirmTeleport, SPlayerCommand, SPlayerPosition,
            SPlayerPositionRotation, SPlayerRotation, SSwingArm,
        },
        status::{SPingRequest, SStatusRequest},
    },
    text::TextComponent,
    ClientPacket, ConnectionState, PacketError, RawPacket, ServerPacket,
};

use std::io::Read;
use thiserror::Error;

pub mod authentication;
mod client_packet;
pub mod player_packet;

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

pub struct Client {
    pub player: Option<Player>,

    pub gameprofile: Option<GameProfile>,

    pub config: Option<PlayerConfig>,
    pub brand: Option<String>,

    pub connection_state: ConnectionState,
    pub encrytion: bool,
    pub closed: bool,
    pub token: Rc<Token>,
    pub connection: TcpStream,
    pub address: SocketAddr,
    enc: PacketEncoder,
    dec: PacketDecoder,
    pub client_packets_queue: VecDeque<RawPacket>,
}

impl Client {
    pub fn new(token: Rc<Token>, connection: TcpStream, address: SocketAddr) -> Self {
        Self {
            gameprofile: None,
            config: None,
            brand: None,
            token,
            address,
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
        shared_secret: &[u8], // decrypted
    ) -> Result<(), EncryptionError> {
        self.encrytion = true;
        let crypt_key: [u8; 16] = shared_secret
            .try_into()
            .map_err(|_| EncryptionError::SharedWrongLength)?;
        self.dec.enable_encryption(&crypt_key);
        self.enc.enable_encryption(&crypt_key);
        Ok(())
    }

    pub fn is_player(&self) -> bool {
        self.player.is_some()
    }

    /// Send a Clientbound Packet to the Client
    pub fn send_packet<P: ClientPacket>(&mut self, packet: P) {
        self.enc
            .append_packet(packet)
            .unwrap_or_else(|e| self.kick(&e.to_string()));
        self.connection
            .write_all(&self.enc.take())
            .map_err(|_| PacketError::ConnectionWrite)
            .unwrap_or_else(|e| self.kick(&e.to_string()));
    }

    pub fn try_send_packet<P: ClientPacket>(&mut self, packet: P) -> Result<(), PacketError> {
        self.enc.append_packet(packet)?;
        self.connection
            .write_all(&self.enc.take())
            .map_err(|_| PacketError::ConnectionWrite)?;
        Ok(())
    }

    pub fn teleport(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        assert!(self.is_player());
        // TODO
        let id = 0;
        let player = self.player.as_mut().unwrap();
        let entity = &mut player.entity;
        entity.x = x;
        entity.y = y;
        entity.z = z;
        entity.lastx = x;
        entity.lasty = y;
        entity.lastz = z;
        entity.yaw = yaw;
        entity.pitch = pitch;
        player.awaiting_teleport = Some(id.into());
        self.send_packet(CSyncPlayerPostion::new(x, y, z, yaw, pitch, 0, id.into()));
    }

    pub fn set_gamemode(&mut self, gamemode: GameMode) {
        self.send_packet(CGameEvent::new(3, gamemode.to_f32().unwrap()));
    }

    pub async fn process_packets(&mut self, server: &mut Server) {
        let mut i = 0;
        while i < self.client_packets_queue.len() {
            let mut packet = self.client_packets_queue.remove(i).unwrap();
            self.handle_packet(server, &mut packet).await;
            i += 1;
        }
    }

    /// Handles an incoming decoded Packet
    pub async fn handle_packet(&mut self, server: &mut Server, packet: &mut RawPacket) {
        // TODO: handle each packet's Error instead of calling .unwrap()
        let bytebuf = &mut packet.bytebuf;
        match self.connection_state {
            pumpkin_protocol::ConnectionState::HandShake => match packet.id.0 {
                SHandShake::PACKET_ID => {
                    self.handle_handshake(server, SHandShake::read(bytebuf).unwrap())
                }
                _ => log::error!(
                    "Failed to handle packet id {} while in Handshake state",
                    packet.id.0
                ),
            },
            pumpkin_protocol::ConnectionState::Status => match packet.id.0 {
                SStatusRequest::PACKET_ID => {
                    self.handle_status_request(server, SStatusRequest::read(bytebuf).unwrap())
                }
                SPingRequest::PACKET_ID => {
                    self.handle_ping_request(server, SPingRequest::read(bytebuf).unwrap())
                }
                _ => log::error!(
                    "Failed to handle packet id {} while in Status state",
                    packet.id.0
                ),
            },
            pumpkin_protocol::ConnectionState::Login => match packet.id.0 {
                SLoginStart::PACKET_ID => {
                    self.handle_login_start(server, SLoginStart::read(bytebuf).unwrap())
                }
                SEncryptionResponse::PACKET_ID => self.handle_encryption_response(
                    server,
                    SEncryptionResponse::read(bytebuf).unwrap(),
                ).await,
                SLoginPluginResponse::PACKET_ID => self
                    .handle_plugin_response(server, SLoginPluginResponse::read(bytebuf).unwrap()),
                SLoginAcknowledged::PACKET_ID => self
                    .handle_login_acknowledged(server, SLoginAcknowledged::read(bytebuf).unwrap()),
                _ => log::error!(
                    "Failed to handle packet id {} while in Login state",
                    packet.id.0
                ),
            },
            pumpkin_protocol::ConnectionState::Config => match packet.id.0 {
                SClientInformation::PACKET_ID => self
                    .handle_client_information(server, SClientInformation::read(bytebuf).unwrap()),
                SPluginMessage::PACKET_ID => {
                    self.handle_plugin_message(server, SPluginMessage::read(bytebuf).unwrap())
                }
                SAcknowledgeFinishConfig::PACKET_ID => self.handle_config_acknowledged(
                    server,
                    SAcknowledgeFinishConfig::read(bytebuf).unwrap(),
                ),
                SKnownPacks::PACKET_ID => {
                    self.handle_known_packs(server, SKnownPacks::read(bytebuf).unwrap())
                }
                _ => log::error!(
                    "Failed to handle packet id {} while in Config state",
                    packet.id.0
                ),
            },
            pumpkin_protocol::ConnectionState::Play => {
                if self.player.is_some() {
                    self.handle_play_packet(server, packet);
                } else {
                    // should be impossible
                    self.kick("no player in play state?")
                }
            }
            _ => log::error!("Invalid Connection state {:?}", self.connection_state),
        }
    }

    pub fn handle_play_packet(&mut self, server: &mut Server, packet: &mut RawPacket) {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(server, SConfirmTeleport::read(bytebuf).unwrap())
            }
            SChatCommand::PACKET_ID => {
                self.handle_chat_command(server, SChatCommand::read(bytebuf).unwrap())
            }
            SPlayerPosition::PACKET_ID => {
                self.handle_position(server, SPlayerPosition::read(bytebuf).unwrap())
            }
            SPlayerPositionRotation::PACKET_ID => self
                .handle_position_rotation(server, SPlayerPositionRotation::read(bytebuf).unwrap()),
            SPlayerRotation::PACKET_ID => {
                self.handle_rotation(server, SPlayerRotation::read(bytebuf).unwrap())
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(server, SPlayerCommand::read(bytebuf).unwrap())
            }
            SSwingArm::PACKET_ID => {
                self.handle_swing_arm(server, SSwingArm::read(bytebuf).unwrap())
            }
            _ => log::error!("Failed to handle player packet id {}", packet.id.0),
        }
    }

    // Reads the connection until our buffer of len 4096 is full, then decode
    /// Close connection when an error occurs
    pub async fn poll(&mut self, server: &mut Server, event: &Event) {
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
                            self.process_packets(server).await;
                        }
                    }
                    Err(err) => self.kick(&err.to_string()),
                }
                self.dec.clear();
            }
        }
    }

    pub fn send_system_message(&mut self, text: TextComponent) {
        self.send_packet(CSystemChatMessge::new(text, false));
    }

    /// Kicks the Client with a reason depending on the connection state
    pub fn kick(&mut self, reason: &str) {
        dbg!(reason);
        match self.connection_state {
            ConnectionState::Login => {
                self.try_send_packet(CLoginDisconnect::new(
                    &serde_json::to_string_pretty(&reason).unwrap(),
                ))
                .unwrap_or_else(|_| self.close());
            }
            ConnectionState::Config => {
                self.try_send_packet(CConfigDisconnect::new(reason))
                    .unwrap_or_else(|_| self.close());
            }
            ConnectionState::Play => {
                self.try_send_packet(CPlayDisconnect::new(TextComponent {
                    text: reason.to_string(),
                }))
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
