#![deny(clippy::all)]
#![warn(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![warn(clippy::cargo)]
// REMOVE SOME WHEN RELEASE
#![expect(clippy::cargo_common_metadata)]
#![expect(clippy::multiple_crate_versions)]
#![expect(clippy::while_float)]
#![expect(clippy::significant_drop_in_scrutinee)]
#![expect(clippy::significant_drop_tightening)]
#![expect(clippy::future_not_send)]
#![expect(clippy::single_call_fn)]
#![expect(clippy::cast_sign_loss)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::cast_possible_wrap)]
#![expect(clippy::too_many_lines)]
#![expect(clippy::missing_panics_doc)]
#![expect(clippy::missing_errors_doc)]
#![expect(clippy::module_name_repetitions)]
#![expect(clippy::struct_excessive_bools)]
#![expect(clippy::many_single_char_names)]
#![expect(clippy::float_cmp)]

#[cfg(target_os = "wasi")]
compile_error!("Compiling for WASI targets is not supported!");

use log::LevelFilter;

use client::Client;
use server::{ticker::Ticker, Server};
use std::io::{self};
use tokio::io::{AsyncBufReadExt, BufReader};

// Setup some tokens to allow us to identify which event is for which socket.

pub mod client;
pub mod commands;
pub mod entity;
pub mod error;
pub mod proxy;
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

#[tokio::main]
async fn main() -> io::Result<()> {
    use std::sync::Arc;

    use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
    use pumpkin_core::text::{color::NamedColor, TextComponent};
    use rcon::RCONServer;
    use std::time::Instant;

    init_logger();

    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap();

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
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        // TODO: Gracefully exit?
        std::process::exit(1);
    }));

    let time = Instant::now();

    // Setup the TCP server socket.
    let addr = BASIC_CONFIG.server_address;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to start TcpListener");

    let use_console = ADVANCED_CONFIG.commands.use_console;
    let rcon = ADVANCED_CONFIG.rcon.clone();

    let server = Arc::new(Server::new());
    let mut ticker = Ticker::new(BASIC_CONFIG.tps);

    log::info!("Started Server took {}ms", time.elapsed().as_millis());
    log::info!("You now can connect to the server, Listening on {}", addr);

    if use_console {
        let server = server.clone();
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
                        .handle_command(&mut commands::CommandSender::Console, &server, &out)
                        .await;
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
    {
        let server = server.clone();
        tokio::spawn(async move {
            ticker.run(&server).await;
        });
    }
    let mut unique_id = 0;
    loop {
        // Asynchronously wait for an inbound socket.
        let (connection, address) = listener.accept().await?;

        if let Err(e) = connection.set_nodelay(true) {
            log::warn!("failed to set TCP_NODELAY {e}");
        }

        unique_id += 1;
        let id = unique_id;

        log::info!(
            "Accepted connection from: {} (id: {})",
            scrub_address(&format!("{address}")),
            id
        );

        let client = Arc::new(Client::new(id, connection, addr));

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
                let id = client.id;
                log::debug!("Creating player for id {}", id);
                let (player, world) = server.add_player(id, client).await;
                world.spawn_player(&BASIC_CONFIG, player.clone()).await;
                // poll Player
                while !player
                    .client
                    .closed
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    let open = player.client.poll().await;
                    if open {
                        player.process_packets(&server).await;
                    };
                }
                player.remove().await;
            }
        });
    }
}
