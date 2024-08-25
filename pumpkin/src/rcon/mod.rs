use std::{
    collections::HashMap,
    io::{self, Read},
    sync::Arc,
};

use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use packet::{Packet, PacketError, PacketType};
use thiserror::Error;

use crate::{commands::handle_command, config::RCONConfig, server::Server};

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

const SERVER: Token = Token(0);

pub struct RCONServer {}

impl RCONServer {
    pub async fn new(
        config: &RCONConfig,
        server: Arc<tokio::sync::Mutex<Server>>,
    ) -> Result<Self, io::Error> {
        assert!(config.enabled, "RCON is not enabled");
        let addr = format!("{}:{}", config.ip, config.port)
            .parse()
            .expect("Failed to parse RCON address");
        let mut poll = Poll::new().unwrap();
        let mut listener = TcpListener::bind(addr).unwrap();

        poll.registry()
            .register(&mut listener, SERVER, Interest::READABLE)
            .unwrap();

        let mut unique_token = Token(SERVER.0 + 1);

        let mut events = Events::with_capacity(20);

        let mut connections: HashMap<Token, RCONClient> = HashMap::new();

        let password = config.password.clone();

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    SERVER => loop {
                        // Received an event for the TCP server socket, which
                        // indicates we can accept an connection.
                        let (mut connection, address) = match listener.accept() {
                            Ok((connection, address)) => (connection, address),
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // If we get a `WouldBlock` error we know our
                                // listener has no more incoming connections queued,
                                // so we can return to polling and wait for some
                                // more.
                                break;
                            }
                            Err(e) => {
                                // If it was any other kind of error, something went
                                // wrong and we terminate with an error.
                                return Err(e);
                            }
                        };
                        log::info!("Accepted connection from: {}", address);

                        let token = Self::next(&mut unique_token);
                        poll.registry()
                            .register(
                                &mut connection,
                                token,
                                Interest::READABLE.add(Interest::WRITABLE),
                            )
                            .unwrap();
                        connections.insert(token, RCONClient::new(connection));
                    },

                    token => {
                        let done = if let Some(client) = connections.get_mut(&token) {
                            client.handle(&server, &password).await
                        } else {
                            false
                        };
                        if done {
                            if let Some(mut client) = connections.remove(&token) {
                                poll.registry().deregister(&mut client.connection)?;
                            }
                        }
                    }
                }
            }
        }
    }

    fn next(current: &mut Token) -> Token {
        let next = current.0;
        current.0 += 1;
        Token(next)
    }
}

pub struct RCONClient {
    connection: TcpStream,
    logged_in: bool,
    incoming: Vec<u8>,
    closed: bool,
}

impl RCONClient {
    pub fn new(connection: TcpStream) -> Self {
        Self {
            connection,
            logged_in: false,
            incoming: Vec::new(),
            closed: false,
        }
    }

    pub async fn handle(
        &mut self,
        server: &Arc<tokio::sync::Mutex<Server>>,
        password: &str,
    ) -> bool {
        if !self.closed {
            loop {
                match self.read_bytes() {
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
            match self.poll(server, password).await {
                Ok(()) => {}
                Err(e) => {
                    log::error!("rcon error: {e}");
                    self.closed = true;
                }
            }
        }
        self.closed
    }

    async fn poll(
        &mut self,
        server: &Arc<tokio::sync::Mutex<Server>>,
        password: &str,
    ) -> Result<(), PacketError> {
        loop {
            let packet = match self.receive_packet().await? {
                Some(p) => p,
                None => return Ok(()),
            };

            match packet.get_type() {
                PacketType::Auth => {
                    let body = packet.get_body();
                    if !body.is_empty() && packet.get_body() == password {
                        self.send(&mut Packet::new(
                            packet.get_id(),
                            PacketType::AuthResponse,
                            "".into(),
                        ))
                        .await
                        .unwrap();
                        log::info!("RCON Client logged in successfully");
                        self.logged_in = true;
                    } else {
                        log::warn!("RCON Client has tried wrong password");
                        self.send(&mut Packet::new(-1, PacketType::AuthResponse, "".into()))
                            .await
                            .unwrap();
                        return Err(PacketError::WrongPassword);
                    }
                }
                PacketType::ExecCommand => {
                    if self.logged_in {
                        let mut output = Vec::new();
                        let mut server = server.lock().await;
                        handle_command(
                            &mut crate::commands::CommandSender::Rcon(&mut output),
                            &mut server,
                            packet.get_body(),
                        );
                        for line in output {
                            self.send(&mut Packet::new(packet.get_id(), PacketType::Output, line))
                                .await
                                .unwrap();
                        }
                    }
                }
                PacketType::Output => todo!(),
                PacketType::AuthResponse => unreachable!(),
            }
        }
    }

    fn read_bytes(&mut self) -> io::Result<bool> {
        let mut buf = [0; 1460];
        let n = self.connection.read(&mut buf)?;
        if n == 0 {
            return Ok(true);
        }
        self.incoming.extend_from_slice(&buf[..n]);
        Ok(false)
    }

    async fn send(&mut self, packet: &mut Packet) -> io::Result<()> {
        packet.send_packet(&mut self.connection).await
    }

    async fn receive_packet(&mut self) -> Result<Option<Packet>, PacketError> {
        Packet::deserialize(&mut self.incoming).await
    }
}
