use log::warn;
use logging::LoggingConfig;
use pumpkin_core::{Difficulty, GameMode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// TODO: when https://github.com/rust-lang/rfcs/pull/3681 gets merged, replace serde-inline-default with native syntax
use serde_inline_default::serde_inline_default;

use std::{
    fs,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::LazyLock,
};

pub mod auth;
pub mod logging;
pub mod proxy;
pub mod resource_pack;

pub use auth::AuthenticationConfig;
pub use commands::CommandsConfig;
pub use compression::CompressionConfig;
pub use pvp::PVPConfig;
pub use rcon::RCONConfig;

mod commands;
pub mod compression;
mod pvp;
mod rcon;

use proxy::ProxyConfig;
use resource_pack::ResourcePackConfig;

pub static ADVANCED_CONFIG: LazyLock<AdvancedConfiguration> =
    LazyLock::new(AdvancedConfiguration::load);

pub static BASIC_CONFIG: LazyLock<BasicConfiguration> = LazyLock::new(BasicConfiguration::load);

/// The idea is that Pumpkin should very customizable.
/// You can Enable or Disable Features depending on your needs.
///
/// This also allows you get some Performance or Resource boosts.
/// Important: The Configuration should match Vanilla by default
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AdvancedConfiguration {
    pub proxy: ProxyConfig,
    pub authentication: AuthenticationConfig,
    pub packet_compression: CompressionConfig,
    pub resource_pack: ResourcePackConfig,
    pub commands: CommandsConfig,
    pub rcon: RCONConfig,
    pub pvp: PVPConfig,
    pub logging: LoggingConfig,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct BasicConfiguration {
    /// The address to bind the server to.
    #[serde(default = "default_server_address")]
    pub server_address: SocketAddr,
    /// The seed for world generation.
    #[serde(default = "String::new")]
    pub seed: String,
    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    #[serde_inline_default(10000)]
    pub max_players: u32,
    /// The maximum view distance for players.
    #[serde_inline_default(10)]
    pub view_distance: u8,
    /// The maximum simulated view distance.
    #[serde_inline_default(10)]
    pub simulation_distance: u8,
    /// The default game difficulty.
    #[serde_inline_default(Difficulty::Normal)]
    pub default_difficulty: Difficulty,
    /// Whether the Nether dimension is enabled.
    #[serde_inline_default(true)]
    pub allow_nether: bool,
    /// Whether the server is in hardcore mode.
    #[serde_inline_default(false)]
    pub hardcore: bool,
    /// Whether online mode is enabled. Requires valid Minecraft accounts.
    #[serde_inline_default(true)]
    pub online_mode: bool,
    /// Whether packet encryption is enabled. Required when online mode is enabled.
    #[serde_inline_default(true)]
    pub encryption: bool,
    /// The server's description displayed on the status screen.
    #[serde_inline_default("A Blazing fast Pumpkin Server!".to_string())]
    pub motd: String,
    #[serde_inline_default(20.0)]
    pub tps: f32,
    /// The default game mode for players.
    #[serde_inline_default(GameMode::Survival)]
    pub default_gamemode: GameMode,
    /// Whether to remove IPs from logs or not
    #[serde_inline_default(true)]
    pub scrub_ips: bool,
    /// Whether to use a server favicon
    #[serde_inline_default(true)]
    pub use_favicon: bool,
    /// Path to server favicon
    #[serde_inline_default("icon.png".to_string())]
    pub favicon_path: String,
}

fn default_server_address() -> SocketAddr {
    SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25565)
}

impl Default for BasicConfiguration {
    fn default() -> Self {
        Self {
            server_address: default_server_address(),
            seed: "".to_string(),
            max_players: 100000,
            view_distance: 10,
            simulation_distance: 10,
            default_difficulty: Difficulty::Normal,
            allow_nether: true,
            hardcore: false,
            online_mode: true,
            encryption: true,
            motd: "A Blazing fast Pumpkin Server!".to_string(),
            tps: 20.0,
            default_gamemode: GameMode::Survival,
            scrub_ips: true,
            use_favicon: true,
            favicon_path: "icon.png".to_string(),
        }
    }
}

trait LoadConfiguration {
    fn load() -> Self
    where
        Self: Sized + Default + Serialize + DeserializeOwned,
    {
        let path = Self::get_path();

        let config = if path.exists() {
            let file_content = fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("Couldn't read configuration file at {:?}", path));

            toml::from_str(&file_content).unwrap_or_else(|err| {
                panic!(
                    "Couldn't parse config at {:?}. Reason: {}. This is is proberbly caused by an Config update, Just delete the old Config and start Pumpkin again",
                    path,
                    err.message()
                )
            })
        } else {
            let content = Self::default();

            if let Err(err) = fs::write(path, toml::to_string(&content).unwrap()) {
                warn!(
                    "Couldn't write default config to {:?}. Reason: {}. This is is proberbly caused by an Config update, Just delete the old Config and start Pumpkin again",
                    path, err
                );
            }

            content
        };

        config.validate();
        config
    }

    fn get_path() -> &'static Path;

    fn validate(&self);
}

impl LoadConfiguration for AdvancedConfiguration {
    #[cfg(not(target_os = "android"))]
    fn get_path() -> &'static Path {
        Path::new("features.toml")
    }

    #[cfg(target_os = "android")]
    fn get_path() -> &'static Path {
        Path::new("/storage/emulated/0/Documents/Pumpkin/features.toml")
    }

    fn validate(&self) {
        self.resource_pack.validate()
    }
}

impl LoadConfiguration for BasicConfiguration {
    #[cfg(not(target_os = "android"))]
    fn get_path() -> &'static Path {
        Path::new("configuration.toml")
    }

    #[cfg(target_os = "android")]
    fn get_path() -> &'static Path {
        Path::new("/storage/emulated/0/Documents/Pumpkin/configuration.toml")
    }

    fn validate(&self) {
        assert!(self.view_distance >= 2, "View distance must be at least 2");
        assert!(
            self.view_distance <= 32,
            "View distance must be less than 32"
        );
        if self.online_mode {
            assert!(
                self.encryption,
                "When Online Mode is enabled, Encryption must be enabled"
            )
        }
    }
}
