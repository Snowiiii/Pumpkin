use client::Client;
use pumpkin_core::text::TextComponent;

pub mod client;
pub mod command;
pub mod entity;
pub mod error;
pub mod plugin;
pub mod proxy;
pub mod server;
pub mod world;

const GIT_VERSION: &str = env!("GIT_VERSION");
