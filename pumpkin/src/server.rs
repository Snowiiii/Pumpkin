use std::io::Cursor;

use base64::{engine::general_purpose, Engine};

use crate::protocol::{Players, Sample, StatusResponse, Version};

pub struct Server {
    pub status_response: StatusResponse,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

        let status_response = StatusResponse {
            version: Version {
                name: "1.21".into(),
                protocol: 767,
            },
            players: Players {
                max: 20,
                online: 0,
                sample: Sample {
                    name: "".into(),
                    id: "".into(),
                },
            },
            description: "Pumpkin Server".into(),
            favicon: Self::load_icon(path),
        };
        Self { status_response }
    }

    pub fn load_icon(path: &str) -> String {
        let mut icon = match image::open(path).map_err(|e| panic!("error loading icon: {}", e)) {
            Ok(icon) => icon,
            Err(_) => return "".into(),
        };
        icon = icon.resize_exact(64, 64, image::imageops::FilterType::Triangle);
        let mut image = Vec::new();
        icon.write_to(&mut Cursor::new(&mut image), image::ImageFormat::Png)
            .unwrap();
        let mut result = "data:image/png;base64,".to_owned();
        general_purpose::STANDARD.encode_string(image, &mut result);
        result
    }
}
