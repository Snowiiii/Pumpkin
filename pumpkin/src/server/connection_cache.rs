use core::error;
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

use base64::{engine::general_purpose, Engine as _};
use pumpkin_config::{BasicConfiguration, BASIC_CONFIG};
use pumpkin_protocol::{
    client::{config::CPluginMessage, status::CStatusResponse},
    Players, StatusResponse, VarInt, Version, CURRENT_MC_PROTOCOL,
};

use super::CURRENT_MC_VERSION;

const DEFAULT_ICON: &[u8] = include_bytes!("../../../assets/default_icon.png");

fn load_icon_from_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn error::Error>> {
    let mut icon_file = File::open(path)?;
    let mut buf = Vec::new();
    icon_file.read_to_end(&mut buf)?;
    load_icon_from_bytes(&buf)
}

fn load_icon_from_bytes(png_data: &[u8]) -> Result<String, Box<dyn error::Error>> {
    let icon = png::Decoder::new(Cursor::new(&png_data));
    let reader = icon.read_info()?;
    let info = reader.info();
    assert!(info.width == 64, "Icon width must be 64");
    assert!(info.height == 64, "Icon height must be 64");

    // Reader consumes the image. Once we verify dimensions, we want to encode the entire raw image
    let mut result = "data:image/png;base64,".to_owned();
    general_purpose::STANDARD.encode_string(png_data, &mut result);
    Ok(result)
}

pub struct CachedStatus {
    status_response: StatusResponse,
    // We cache the json response here so we don't parse it every time someone makes a Status request.
    // Keep in mind that we must parse this again, when the StatusResponse changes which usually happen when a player joins or leaves
    status_response_json: String,
}

pub struct CachedBranding {
    /// Cached Server brand buffer so we don't have to rebuild them every time a player joins
    cached_server_brand: Vec<u8>,
}

impl CachedBranding {
    pub fn new() -> Self {
        let cached_server_brand = Self::build_brand();
        Self {
            cached_server_brand,
        }
    }
    pub fn get_branding(&self) -> CPluginMessage {
        CPluginMessage::new("minecraft:brand", &self.cached_server_brand)
    }
    fn build_brand() -> Vec<u8> {
        let brand = "Pumpkin";
        let mut buf = Vec::new();
        VarInt(brand.len() as i32).encode(&mut buf);
        buf.extend_from_slice(brand.as_bytes());
        buf
    }
}

impl CachedStatus {
    pub fn new() -> Self {
        let status_response = Self::build_response(&BASIC_CONFIG);
        let status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");

        Self {
            status_response,
            status_response_json,
        }
    }

    pub fn get_status(&self) -> CStatusResponse<'_> {
        CStatusResponse::new(&self.status_response_json)
    }

    // TODO: Player samples
    pub fn add_player(&mut self) {
        let status_response = &mut self.status_response;
        if let Some(players) = &mut status_response.players {
            players.online += 1;
        }

        self.status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");
    }

    pub fn remove_player(&mut self) {
        let status_response = &mut self.status_response;
        if let Some(players) = &mut status_response.players {
            players.online -= 1;
        }

        self.status_response_json = serde_json::to_string(&status_response)
            .expect("Failed to parse Status response into JSON");
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let icon = if config.use_favicon {
            let icon_path = &config.favicon_path;
            log::debug!("Loading server favicon from '{}'", icon_path);
            match load_icon_from_file(icon_path).or_else(|err| {
                if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        log::info!("Favicon '{}' not found; using default icon.", icon_path);
                    } else {
                        log::error!(
                            "Unable to load favicon at '{}': I/O error - {}; using default icon.",
                            icon_path,
                            io_err
                        );
                    }
                } else {
                    log::error!(
                        "Unable to load favicon at '{}': other error - {}; using default icon.",
                        icon_path,
                        err
                    );
                }
                load_icon_from_bytes(DEFAULT_ICON)
            }) {
                Ok(result) => Some(result),
                Err(err) => {
                    log::warn!("Failed to load default icon: {}", err);
                    None
                }
            }
        } else {
            log::info!("Not using a server favicon");
            None
        };

        StatusResponse {
            version: Some(Version {
                name: CURRENT_MC_VERSION.into(),
                protocol: CURRENT_MC_PROTOCOL,
            }),
            players: Some(Players {
                max: config.max_players,
                online: 0,
                sample: vec![],
            }),
            description: config.motd.clone(),
            favicon: icon,
            enforce_secure_chat: false,
        }
    }
}

impl Default for CachedStatus {
    fn default() -> Self {
        Self::new()
    }
}
