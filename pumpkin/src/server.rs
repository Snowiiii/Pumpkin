use std::io::Cursor;

use base64::{engine::general_purpose, Engine};
use rsa::{rand_core::OsRng, traits::PublicKeyParts, RsaPrivateKey, RsaPublicKey};

use crate::protocol::{Players, Sample, StatusResponse, Version};

pub struct Server {
    pub compression_threshold: Option<u8>,

    pub online_mode: bool,
    pub encriyption: bool, // encription is always required when online_mode is disabled
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
    pub public_key_der: Box<[u8]>,

    pub status_response: StatusResponse,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        let status_response = Self::default_response();

        // todo, only create when needed
        let (public_key, private_key) = Self::generate_keys();

        let public_key_der = rsa_der::public_key_to_der(
            &private_key.n().to_bytes_be(),
            &private_key.e().to_bytes_be(),
        )
        .into_boxed_slice();

        Self {
            online_mode: true,
            encriyption: true,
            compression_threshold: None, // 256
            public_key,
            private_key,
            status_response,
            public_key_der,
        }
    }

    pub fn default_response() -> StatusResponse {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

        StatusResponse {
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
        }
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

    pub fn generate_keys() -> (RsaPublicKey, RsaPrivateKey) {
        let priv_key = RsaPrivateKey::new(&mut OsRng, 1024).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        (pub_key, priv_key)
    }
}
