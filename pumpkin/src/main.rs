#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![deny(clippy::cargo)]
#![deny(clippy::if_then_some_else_none)]
// REMOVE SOME WHEN RELEASE
#![expect(clippy::cargo_common_metadata)]
#![expect(clippy::multiple_crate_versions)]
#![expect(clippy::single_call_fn)]
#![expect(clippy::cast_sign_loss)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::cast_possible_wrap)]
#![expect(clippy::missing_panics_doc)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::struct_excessive_bools)]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

use log::LevelFilter;

use client::Client;
use server::{ticker::Ticker, Server};
use std::io::{self};
use tokio::io::{AsyncBufReadExt, BufReader};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

use std::sync::Arc;

use crate::server::CURRENT_MC_VERSION;
use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_core::text::{color::NamedColor, TextComponent};
use pumpkin_protocol::CURRENT_MC_PROTOCOL;
use rcon::RCONServer;
use std::time::Instant;
// Setup some tokens to allow us to identify which event is for which socket.

pub mod block;
pub mod client;
pub mod command;
pub mod entity;
pub mod error;
pub mod lan_broadcast;
pub mod proxy;
pub mod query;
pub mod rcon;
pub mod server;
pub mod world;

fn scrub_address(ip: &str) -> String {
    use pumpkin_config::BASIC_CONFIG;
    if BASIC_CONFIG.scrub_ips {
        ip.chars()
            .map(|ch| if ch == '.' || ch == ':' { ch } else { 'x' })
            .collect()
    } else {
        ip.to_string()
    }
}

fn init_logger() {
    use pumpkin_config::ADVANCED_CONFIG;
    if ADVANCED_CONFIG.logging.enabled {
        let mut logger = simple_logger::SimpleLogger::new();
        logger = logger.with_timestamp_format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ));

        if !ADVANCED_CONFIG.logging.timestamp {
            logger = logger.without_timestamps();
        }

        if ADVANCED_CONFIG.logging.env {
            logger = logger.env();
        }

        logger = logger.with_level(convert_logger_filter(ADVANCED_CONFIG.logging.level));

        logger = logger.with_colors(ADVANCED_CONFIG.logging.color);
        logger = logger.with_threads(ADVANCED_CONFIG.logging.threads);
        logger.init().unwrap();
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

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_VERSION: &str = env!("GIT_VERSION");

// WARNING: All rayon calls from the tokio runtime must be non-blocking! This includes things
// like `par_iter`. These should be spawned in the the rayon pool and then passed to the tokio
// runtime with a channel! See `Level::fetch_chunks` as an example!
#[tokio::main]
#[expect(clippy::too_many_lines)]
async fn main() {
    let time = Instant::now();
    init_logger();

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        // TODO: Gracefully exit?
        std::process::exit(1);
    }));

    log::info!("Starting Pumpkin {CARGO_PKG_VERSION} ({GIT_VERSION}) for Minecraft {CURRENT_MC_VERSION} (Protocol {CURRENT_MC_PROTOCOL})",);

    log::debug!(
        "Build info: FAMILY: \"{}\", OS: \"{}\", ARCH: \"{}\", BUILD: \"{}\"",
        std::env::consts::FAMILY,
        std::env::consts::OS,
        std::env::consts::ARCH,
        if cfg!(debug_assertions) {
            "Debug"
        } else {
            "Release"
        }
    );

    log::warn!("Pumpkin is currently under heavy development!");
    log::info!("Report Issues on https://github.com/Snowiiii/Pumpkin/issues");
    log::info!("Join our Discord for community support https://discord.com/invite/wT8XjrjKkf");

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect("Unable to setup signal handlers");
    });

    // Setup the TCP server socket.
    let listener = tokio::net::TcpListener::bind(BASIC_CONFIG.server_address)
        .await
        .expect("Failed to start TcpListener");
    // In the event the user puts 0 for their port, this will allow us to know what port it is running on
    let addr = listener
        .local_addr()
        .expect("Unable to get the address of server!");

    let use_console = ADVANCED_CONFIG.commands.use_console;
    let rcon = ADVANCED_CONFIG.rcon.clone();

    let server = Arc::new(Server::new());
    let mut ticker = Ticker::new(BASIC_CONFIG.tps);

    log::info!("Started Server took {}ms", time.elapsed().as_millis());
    log::info!("You now can connect to the server, Listening on {}", addr);

    if use_console {
        setup_console(server.clone());
    }
    if rcon.enabled {
        let server = server.clone();
        tokio::spawn(async move {
            RCONServer::new(&rcon, server).await.unwrap();
        });
    }

    if ADVANCED_CONFIG.query.enabled {
        log::info!("Query protocol enabled. Starting...");
        tokio::spawn(query::start_query_handler(server.clone(), addr));
    }

    if ADVANCED_CONFIG.lan_broadcast.enabled {
        log::info!("LAN broadcast enabled. Starting...");
        tokio::spawn(lan_broadcast::start_lan_broadcast(addr));
    }

    {
        let server = server.clone();
        tokio::spawn(async move {
            ticker.run(&server).await;
        });
    }

    let mut master_client_id: u16 = 0;
    loop {
        // Asynchronously wait for an inbound socket.
        let (connection, address) = listener.accept().await.unwrap();

        if let Err(e) = connection.set_nodelay(true) {
            log::warn!("failed to set TCP_NODELAY {e}");
        }

        let id = master_client_id;
        master_client_id = master_client_id.wrapping_add(1);

        log::info!(
            "Accepted connection from: {} (id {})",
            scrub_address(&format!("{address}")),
            id
        );

        let client = Arc::new(Client::new(connection, addr, id));

        let server = server.clone();
        tokio::spawn(async move {
            while !client.closed.load(std::sync::atomic::Ordering::Relaxed)
                && !client
                    .make_player
                    .load(std::sync::atomic::Ordering::Relaxed)
            {
                let open = client.poll().await;
                if open {
                    client.process_packets(&server).await;
                };
            }
            if client
                .make_player
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let (player, world) = server.add_player(client).await;
                world
                    .spawn_player(&BASIC_CONFIG, player.clone(), &server)
                    .await;

                // poll Player
                while !player
                    .client
                    .closed
                    .load(core::sync::atomic::Ordering::Relaxed)
                {
                    let open = player.client.poll().await;
                    if open {
                        player.process_packets(&server).await;
                    };
                }
                log::debug!("Cleaning up player for id {}", id);
                player.remove().await;
                server.remove_player().await;
            }
        });
    }
}

fn handle_interrupt() {
    log::warn!(
        "{}",
        TextComponent::text("Received interrupt signal; stopping server...")
            .color_named(NamedColor::Red)
            .to_pretty_console()
    );
    std::process::exit(0);
}

// Non-UNIX Ctrl-C handling
#[cfg(not(unix))]
async fn setup_sighandler() -> io::Result<()> {
    if ctrl_c().await.is_ok() {
        handle_interrupt();
    }

    Ok(())
}

// Unix signal handling
#[cfg(unix)]
async fn setup_sighandler() -> io::Result<()> {
    if signal(SignalKind::interrupt())?.recv().await.is_some() {
        handle_interrupt();
    }

    if signal(SignalKind::hangup())?.recv().await.is_some() {
        handle_interrupt();
    }

    if signal(SignalKind::terminate())?.recv().await.is_some() {
        handle_interrupt();
    }

    Ok(())
}

fn setup_console(server: Arc<Server>) {
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        loop {
            let mut out = String::new();

            reader
                .read_line(&mut out)
                .await
                .expect("Failed to read console line");

            if !out.is_empty() {
                let dispatcher = server.command_dispatcher.clone();
                dispatcher
                    .handle_command(&mut command::CommandSender::Console, &server, &out)
                    .await;
            }
        }
    });
}
