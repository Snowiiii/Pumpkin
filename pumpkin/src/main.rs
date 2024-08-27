#![allow(clippy::await_holding_refcell_ref)]
#![allow(clippy::await_holding_lock)]

use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::io::{self};

use client::Client;
use commands::handle_command;
use config::AdvancedConfiguration;

use std::collections::HashMap;

use client::interrupted;
use config::BasicConfiguration;
use server::Server;

// Setup some tokens to allow us to identify which event is for which socket.

pub mod client;
pub mod commands;
pub mod config;
pub mod entity;
pub mod proxy;
pub mod rcon;
pub mod server;
pub mod util;

#[cfg(not(target_os = "wasi"))]
fn main() -> io::Result<()> {
    use std::sync::{Arc, Mutex};

    use entity::player::Player;
    use pumpkin_core::text::{color::NamedColor, TextComponent};
    use rcon::RCONServer;

    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    ctrlc::set_handler(|| {
        log::warn!(
            "{}",
            TextComponent::text("Stopping Server")
                .color_named(NamedColor::Red)
                .to_pretty_console()
        );
        std::process::exit(0);
    })
    .unwrap();
    // ensure rayon is built outside of tokio scope
    rayon::ThreadPoolBuilder::new().build_global().unwrap();
    rt.block_on(async {
        const SERVER: Token = Token(0);
        use std::time::Instant;

        let time = Instant::now();
        let basic_config = BasicConfiguration::load("configuration.toml");

        let advanced_configuration = AdvancedConfiguration::load("features.toml");

        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(128);

        // Setup the TCP server socket.

        let addr = format!(
            "{}:{}",
            basic_config.server_address, basic_config.server_port
        )
        .parse()
        .unwrap();

        let mut listener = TcpListener::bind(addr)?;

        // Register the server with poll we can receive events for it.
        poll.registry()
            .register(&mut listener, SERVER, Interest::READABLE)?;

        // Unique token for each incoming connection.
        let mut unique_token = Token(SERVER.0 + 1);

        let use_console = advanced_configuration.commands.use_console;
        let rcon = advanced_configuration.rcon.clone();

        let mut clients: HashMap<Token, Client> = HashMap::new();
        let mut players: HashMap<Arc<Token>, Arc<Mutex<Player>>> = HashMap::new();

        let server = Arc::new(tokio::sync::Mutex::new(Server::new((
            basic_config,
            advanced_configuration,
        ))));
        log::info!("Started Server took {}ms", time.elapsed().as_millis());
        log::info!("You now can connect to the server, Listening on {}", addr);

        let server1 = server.clone();
        if use_console {
            tokio::spawn(async move {
                let stdin = std::io::stdin();
                loop {
                    let mut out = String::new();
                    stdin
                        .read_line(&mut out)
                        .expect("Failed to read console line");

                    if !out.is_empty() {
                        let mut server = server1.lock().await;
                        handle_command(&mut commands::CommandSender::Console, &mut server, &out);
                    }
                }
            });
        }
        if rcon.enabled {
            let server = server.clone();
            tokio::spawn(async move {
                RCONServer::new(&rcon, server).await.unwrap();
            });
        }
        loop {
            if let Err(err) = poll.poll(&mut events, None) {
                if interrupted(&err) {
                    continue;
                }
                return Err(err);
            }

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
                        if let Err(e) = connection.set_nodelay(true) {
                            log::warn!("failed to set TCP_NODELAY {e}");
                        }

                        log::info!("Accepted connection from: {}", address);

                        let token = next(&mut unique_token);
                        poll.registry().register(
                            &mut connection,
                            token,
                            Interest::READABLE.add(Interest::WRITABLE),
                        )?;
                        let rc_token = Arc::new(token);
                        let client = Client::new(Arc::clone(&rc_token), connection, addr);
                        clients.insert(token, client);
                    },

                    token => {
                        // Poll Players
                        let done = if let Some(player) = players.get_mut(&token) {
                            let mut player = player.lock().unwrap();
                            player.client.poll(event).await;
                            let mut server = server.lock().await;
                            player.process_packets(&mut server);
                            player.client.closed
                        } else {
                            false
                        };

                        if done {
                            if let Some(player) = players.remove(&token) {
                                let mut server = server.lock().await;
                                server.remove_player(&token);
                                let mut player = player.lock().unwrap();
                                poll.registry().deregister(&mut player.client.connection)?;
                            }
                        }

                        // Poll current Clients (non players)
                        // Maybe received an event for a TCP connection.
                        let (done, make_player) = if let Some(client) = clients.get_mut(&token) {
                            client.poll(event).await;
                            let mut server = server.lock().await;
                            client.process_packets(&mut server).await;
                            (client.closed, client.make_player)
                        } else {
                            // Sporadic events happen, we can safely ignore them.
                            (false, false)
                        };
                        if done || make_player {
                            if let Some(mut client) = clients.remove(&token) {
                                if done {
                                    poll.registry().deregister(&mut client.connection)?;
                                } else if make_player {
                                    let token = client.token.clone();
                                    let mut server = server.lock().await;
                                    let player = server.add_player(token.clone(), client);
                                    players.insert(token, player.clone());
                                    let mut player = player.lock().unwrap();
                                    server.spawn_player(&mut player).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

#[cfg(target_os = "wasi")]
fn main() {
    panic!("can't bind to an address with wasi")
}
