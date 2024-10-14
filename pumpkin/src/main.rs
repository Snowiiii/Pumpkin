#![deny(clippy::all)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
// expect
#![expect(clippy::cargo_common_metadata)]
#![expect(clippy::multiple_crate_versions)]
#![expect(clippy::while_float)]
#![expect(clippy::significant_drop_in_scrutinee)]
#![expect(clippy::significant_drop_tightening)]
#![expect(clippy::future_not_send)]
#![expect(clippy::single_call_fn)]
#![expect(clippy::await_holding_lock)]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

use log::LevelFilter;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};

use client::{interrupted, Client};
use pumpkin_protocol::client::play::CKeepAlive;
use pumpkin_protocol::ConnectionState;
use server::Server;
use std::collections::HashMap;
use std::io::{self, Read};
use std::time::Duration;

// Setup some tokens to allow us to identify which event is for which socket.

pub mod client;
pub mod commands;
pub mod entity;
pub mod error;
pub mod proxy;
pub mod rcon;
pub mod server;
pub mod world;

fn init_logger() {
    use pumpkin_config::ADVANCED_CONFIG;
    if ADVANCED_CONFIG.logging.enabled {
        let mut logger = simple_logger::SimpleLogger::new();

        if !ADVANCED_CONFIG.logging.timestamp {
            logger = logger.without_timestamps();
        }

        if ADVANCED_CONFIG.logging.env {
            logger = logger.env();
        }

        logger = logger.with_level(convert_logger_filter(ADVANCED_CONFIG.logging.level));

        logger = logger.with_colors(ADVANCED_CONFIG.logging.color);
        logger = logger.with_threads(ADVANCED_CONFIG.logging.threads);
        logger.init().unwrap()
    }
}

const fn convert_logger_filter(level: pumpkin_config::logging::LevelFilter) -> LevelFilter {
    match level {
        pumpkin_config::logging::LevelFilter::Off => LevelFilter::Off,
        pumpkin_config::logging::LevelFilter::Error => LevelFilter::Error,
        pumpkin_config::logging::LevelFilter::Warn => LevelFilter::Warn,
        pumpkin_config::logging::LevelFilter::Info => LevelFilter::Info,
        pumpkin_config::logging::LevelFilter::Debug => LevelFilter::Debug,
        pumpkin_config::logging::LevelFilter::Trace => LevelFilter::Trace,
    }
}

fn main() -> io::Result<()> {
    use std::sync::Arc;

    use entity::player::Player;
    use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
    use pumpkin_core::text::{color::NamedColor, TextComponent};
    use rcon::RCONServer;

    init_logger();

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

        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(128);

        // Setup the TCP server socket.
        let addr = BASIC_CONFIG.server_address;
        let mut listener = TcpListener::bind(addr)?;

        // Register the server with poll we can receive events for it.
        poll.registry()
            .register(&mut listener, SERVER, Interest::READABLE)?;

        // Unique token for each incoming connection.
        let mut unique_token = Token(SERVER.0 + 1);

        let use_console = ADVANCED_CONFIG.commands.use_console;
        let rcon = ADVANCED_CONFIG.rcon.clone();

        let mut clients: HashMap<Token, Arc<Client>> = HashMap::new();
        let mut players: HashMap<Token, Arc<Player>> = HashMap::new();

        let server = Arc::new(Server::new());
        log::info!("Started Server took {}ms", time.elapsed().as_millis());
        log::info!("You now can connect to the server, Listening on {}", addr);

        if use_console {
            let server = server.clone();
            tokio::spawn(async move {
                let stdin = std::io::stdin();
                loop {
                    let mut out = String::new();
                    stdin
                        .read_line(&mut out)
                        .expect("Failed to read console line");

                    if !out.is_empty() {
                        let dispatcher = server.command_dispatcher.clone();
                        dispatcher.handle_command(
                            &mut commands::CommandSender::Console,
                            &server,
                            &out,
                        );
                    }
                }
            });
        }
        if rcon.enabled {
            let server = server.clone();
            tokio::spawn(async move {
                RCONServer::new(&rcon, &server).await.unwrap();
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
                    s if s == SERVER => loop {
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
                        let keep_alive = tokio::sync::mpsc::channel(1024);
                        let client =
                            Arc::new(Client::new(token, connection, addr, keep_alive.0.into()));

                        {
                            let client = client.clone();
                            let mut receiver = keep_alive.1;
                            tokio::spawn(async move {
                                let mut interval = tokio::time::interval(Duration::from_secs(1));
                                loop {
                                    interval.tick().await;
                                    let now = std::time::Instant::now();
                                    if client.connection_state.load() == ConnectionState::Play {
                                        if now.duration_since(client.last_alive_received.load())
                                            >= Duration::from_secs(15)
                                        {
                                            dbg!("no keep alive");
                                            client.kick("No keep alive received");
                                            break;
                                        }
                                        let random = rand::random::<i64>();
                                        client.send_packet(&CKeepAlive {
                                            keep_alive_id: random,
                                        });
                                        if let Some(id) = receiver.recv().await {
                                            if id == random {
                                                client.last_alive_received.store(now);
                                            }
                                        }
                                    } else {
                                        client.last_alive_received.store(now);
                                    }
                                }
                            });
                        }
                        clients.insert(token, client);
                    },
                    // Maybe received an event for a TCP connection.
                    token => {
                        // poll Player
                        if let Some(player) = players.get_mut(&token) {
                            player.client.poll(event).await;
                            let closed = player
                                .client
                                .closed
                                .load(std::sync::atomic::Ordering::Relaxed);
                            if !closed {
                                player.process_packets(&server).await;
                            }
                            if closed {
                                if let Some(player) = players.remove(&token) {
                                    player.remove().await;
                                    let connection = &mut player.client.connection.lock();
                                    poll.registry().deregister(connection.by_ref())?;
                                }
                            }
                        };

                        // Poll current Clients (non players)
                        let (done, make_player) = if let Some(client) = clients.get_mut(&token) {
                            client.poll(event).await;
                            let closed = client.closed.load(std::sync::atomic::Ordering::Relaxed);
                            if !closed {
                                client.process_packets(&server).await;
                            }
                            (
                                closed,
                                client
                                    .make_player
                                    .load(std::sync::atomic::Ordering::Relaxed),
                            )
                        } else {
                            (false, false)
                        };
                        if done || make_player {
                            if let Some(client) = clients.remove(&token) {
                                if done {
                                    let connection = &mut client.connection.lock();
                                    poll.registry().deregister(connection.by_ref())?;
                                } else if make_player {
                                    let token = client.token;
                                    let (player, world) = server.add_player(token, client).await;
                                    players.insert(token, player.clone());
                                    world.spawn_player(&BASIC_CONFIG, player).await;
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
