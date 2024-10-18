use std::{
    collections::HashMap,
    io::{self},
    net::SocketAddr,
};

use packet::{ClientboundPacket, Packet, PacketError, ServerboundPacket};
use pumpkin_config::{RCONConfig, ADVANCED_CONFIG};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::server::Server;

mod packet;

#[derive(Debug, Error)]
pub enum RCONError {
    #[error("authentication failed")]
    Auth,
    #[error("command exceeds the maximum length")]
    CommandTooLong,
    #[error("{}", _0)]
    Io(io::Error),
}

pub struct RCONServer;

impl RCONServer {
    pub async fn new(config: &RCONConfig, server: &Server) -> Result<Self, io::Error> {
        assert!(config.enabled, "RCON is not enabled");
        let listener = tokio::net::TcpListener::bind(config.address).await.unwrap();

        let mut connections: HashMap<usize, RCONClient> = HashMap::new();

        let password = config.password.clone();

        let mut unique_id = 0;
        loop {
            // Asynchronously wait for an inbound socket.
            let (connection, address) = listener.accept().await?;

            if config.max_connections != 0 && connections.len() >= config.max_connections as usize {
                continue;
            }

            unique_id += 1;
            let token = unique_id;
            connections.insert(token, RCONClient::new(connection, address));

            let done = if let Some(client) = connections.get_mut(&token) {
                client.handle(server, &password).await
            } else {
                false
            };
            if done {
                if let Some(client) = connections.remove(&token) {
                    let config = &ADVANCED_CONFIG.rcon;
                    if config.logging.log_quit {
                        log::info!("RCON ({}): Client closed connection", client.address);
                    }
                }
            }
        }
    }
}

pub struct RCONClient {
    connection: tokio::net::TcpStream,
    address: SocketAddr,
    logged_in: bool,
    incoming: Vec<u8>,
    closed: bool,
}

impl RCONClient {
    pub const fn new(connection: tokio::net::TcpStream, address: SocketAddr) -> Self {
        Self {
            connection,
            address,
            logged_in: false,
            incoming: Vec::new(),
            closed: false,
        }
    }

    pub async fn handle(&mut self, server: &Server, password: &str) -> bool {
        if !self.closed {
            loop {
                match self.read_bytes().await {
                    // Stream closed, so we can't reply, so we just close everything.
                    Ok(true) => return true,
                    Ok(false) => {}
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => {
                        log::error!("could not read packet: {e}");
                        return true;
                    }
                }
            }
            // If we get a close here, we might have a reply, which we still want to write.
            let _ = self.poll(server, password).await.map_err(|e| {
                log::error!("RCON error: {e}");
                self.closed = true;
            });
        }
        self.closed
    }

    async fn poll(&mut self, server: &Server, password: &str) -> Result<(), PacketError> {
        loop {
            let packet = match self.receive_packet().await? {
                Some(p) => p,
                None => return Ok(()),
            };

            let config = &ADVANCED_CONFIG.rcon;
            match packet.get_type() {
                ServerboundPacket::Auth => {
                    let body = packet.get_body();
                    if !body.is_empty() && packet.get_body() == password {
                        self.send(ClientboundPacket::AuthResponse, packet.get_id(), "".into())
                            .await?;
                        if config.logging.log_logged_successfully {
                            log::info!("RCON ({}): Client logged in successfully", self.address);
                        }
                        self.logged_in = true;
                    } else {
                        if config.logging.log_wrong_password {
                            log::info!("RCON ({}): Client has tried wrong password", self.address);
                        }
                        self.send(ClientboundPacket::AuthResponse, -1, "".into())
                            .await?;
                        self.closed = true;
                    }
                }
                ServerboundPacket::ExecCommand => {
                    if self.logged_in {
                        let mut output = Vec::new();
                        let dispatcher = server.command_dispatcher.clone();
                        dispatcher.handle_command(
                            &mut crate::commands::CommandSender::Rcon(&mut output),
                            server,
                            packet.get_body(),
                        );
                        for line in output {
                            if config.logging.log_commands {
                                log::info!("RCON ({}): {}", self.address, line);
                            }
                            self.send(ClientboundPacket::Output, packet.get_id(), line)
                                .await?;
                        }
                    }
                }
            }
        }
    }

    async fn read_bytes(&mut self) -> io::Result<bool> {
        let mut buf = [0; 1460];
        let n = self.connection.read(&mut buf).await?;
        if n == 0 {
            return Ok(true);
        }
        self.incoming.extend_from_slice(&buf[..n]);
        Ok(false)
    }

    async fn send(
        &mut self,
        packet: ClientboundPacket,
        id: i32,
        body: String,
    ) -> Result<(), PacketError> {
        let buf = packet.write_buf(id, body);
        self.connection
            .write(&buf)
            .await
            .map_err(PacketError::FailedSend)?;
        Ok(())
    }

    async fn receive_packet(&mut self) -> Result<Option<Packet>, PacketError> {
        Packet::deserialize(&mut self.incoming).await
    }
}
