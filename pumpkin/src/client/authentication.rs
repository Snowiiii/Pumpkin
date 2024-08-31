use std::{collections::HashMap, net::IpAddr};

use base64::{engine::general_purpose, Engine};
use num_bigint::BigInt;
use pumpkin_config::{auth::TextureConfig, ADVANCED_CONFIG};
use pumpkin_core::ProfileAction;
use pumpkin_protocol::Property;
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::server::Server;

#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct ProfileTextures {
    timestamp: i64,
    profileId: Uuid,
    profileName: String,
    signatureRequired: bool,
    textures: HashMap<String, Texture>,
}

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct Texture {
    url: String,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<Property>,
    #[serde(rename = "profileActions")]
    pub profile_actions: Option<Vec<ProfileAction>>,
}

pub async fn authenticate(
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    server: &mut Server,
) -> Result<GameProfile, AuthError> {
    assert!(ADVANCED_CONFIG.authentication.enabled);
    assert!(server.auth_client.is_some());
    let address = if ADVANCED_CONFIG.authentication.prevent_proxy_connections {
        format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}&ip={ip}")
    } else {
        format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}")
    };
    let auth_client = server
        .auth_client
        .as_ref()
        .ok_or(AuthError::MissingAuthClient)?;

    let response = auth_client
        .get(address)
        .send()
        .await
        .map_err(|_| AuthError::FailedResponse)?;
    match response.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT => Err(AuthError::UnverifiedUsername)?,
        other => Err(AuthError::UnknownStatusCode(other.as_str().to_string()))?,
    }
    let profile: GameProfile = response.json().await.map_err(|_| AuthError::FailedParse)?;
    Ok(profile)
}

pub fn unpack_textures(property: Property, config: &TextureConfig) {
    // TODO: no unwrap
    let from64 = general_purpose::STANDARD.decode(property.value).unwrap();
    let textures: ProfileTextures = serde_json::from_slice(&from64).unwrap();
    for texture in textures.textures {
        is_texture_url_valid(Url::parse(&texture.1.url).unwrap(), config);
    }
}

pub fn auth_digest(bytes: &[u8]) -> String {
    BigInt::from_signed_bytes_be(bytes).to_str_radix(16)
}

pub fn is_texture_url_valid(url: Url, config: &TextureConfig) -> bool {
    let scheme = url.scheme();
    if !config.allowed_url_schemes.contains(&scheme.to_string()) {
        return false;
    }
    let domain = url.domain().unwrap_or("");
    if !config.allowed_url_domains.contains(&domain.to_string()) {
        return false;
    }
    true
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing auth client")]
    MissingAuthClient,
    #[error("Authentication servers are down")]
    FailedResponse,
    #[error("Failed to verify username")]
    UnverifiedUsername,
    #[error("Failed to parse JSON into Game Profile")]
    FailedParse,
    #[error("Unknown Status Code")]
    UnknownStatusCode(String),
}
