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
    Players, Sample, StatusResponse, VarInt, Version, CURRENT_MC_PROTOCOL,
};

use super::CURRENT_MC_VERSION;

pub struct CachedStatus {
    _status_response: StatusResponse,
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
        let mut buf = vec![];
        let _ = VarInt(brand.len() as i32).encode(&mut buf);
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
            _status_response: status_response,
            status_response_json,
        }
    }

    pub fn get_status(&self) -> CStatusResponse<'_> {
        CStatusResponse::new(&self.status_response_json)
    }

    pub fn build_response(config: &BasicConfiguration) -> StatusResponse {
        let icon_path = &config.favicon_path;

        let icon = if icon_path.is_empty() {
            // See if an icon exists at ./icon.png
            let default_local_path = "./icon.png";
            if Path::new(default_local_path).exists() {
                log::info!("Loading server icon from {}", default_local_path);
                let maybe_icon = Self::load_icon(default_local_path);
                match maybe_icon {
                    Ok(result) => Some(result),
                    Err(e) => {
                        log::warn!("Failed to load icon: {:?}", e);
                        None
                    }
                }
            } else {
                log::info!("Using default server icon");
                Some(pumpkin_macros::create_icon!().to_string())
            }
        } else if Path::new(icon_path).exists() {
            log::info!("Loading server icon from {}", icon_path);
            let maybe_icon = Self::load_icon(icon_path);
            match maybe_icon {
                Ok(result) => Some(result),
                Err(e) => {
                    log::warn!("Failed to load icon: {:?}", e);
                    None
                }
            }
        } else {
            // TODO: Add definitive option to have no icon?
            // Currently can just use a bad path
            log::warn!("Failed to load server icon at path {}", icon_path);
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
                sample: vec![Sample {
                    name: "".into(),
                    id: "".into(),
                }],
            }),
            description: config.motd.clone(),
            favicon: icon,
            enforce_secure_chat: false,
        }
    }

    fn load_icon<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn error::Error>> {
        let mut icon_file = File::open(path).expect("Failed to load icon");
        let mut buf = Vec::new();
        icon_file.read_to_end(&mut buf)?;

        let icon = png::Decoder::new(Cursor::new(&buf));
        let reader = icon.read_info()?;
        let info = reader.info();
        assert!(info.width == 64, "Icon width must be 64");
        assert!(info.height == 64, "Icon height must be 64");

        // Reader consumes the image. Once we verify dimensions, we want to encode the entire raw image
        let mut result = "data:image/png;base64,".to_owned();
        general_purpose::STANDARD.encode_string(&buf, &mut result);
        Ok(result)
    }
}
