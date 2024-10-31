#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![deny(clippy::cargo)]
// REMOVE SOME WHEN RELEASE
#![expect(clippy::cargo_common_metadata)]
#![expect(clippy::multiple_crate_versions)]
#![expect(clippy::significant_drop_in_scrutinee)]
#![expect(clippy::significant_drop_tightening)]
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

use pumpkin::{commands, server::Server, server_start};
use pumpkin_core::text::{color::NamedColor, TextComponent};
use std::{
    io::{self},
    sync::Arc,
};
use tokio::io::{AsyncBufReadExt, BufReader};
#[cfg(not(unix))]
use tokio::signal::ctrl_c;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

// Setup some tokens to allow us to identify which event is for which socket.

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
    init_logger();
    // ensure rayon is built outside of tokio scope
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    tokio::spawn(async {
        setup_sighandler()
            .await
            .expect("Unable to setup signal handlers");
    });
    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap();

    server_start(|server| setup_console(server)).await
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
                    .handle_command(&mut commands::CommandSender::Console, &server, &out)
                    .await;
            }
        }
    });
}
