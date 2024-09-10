use std::{collections::HashMap, net::IpAddr, sync::Arc};

use base64::{engine::general_purpose, Engine};
use pumpkin_config::{auth::TextureConfig, ADVANCED_CONFIG};
use pumpkin_core::ProfileAction;
use pumpkin_protocol::Property;
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::server::Server;

#[derive(Deserialize, Clone, Debug)]
#[expect(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTextures {
    timestamp: i64,
    profile_id: Uuid,
    profile_name: String,
    signature_required: bool,
    textures: HashMap<String, Texture>,
}

#[derive(Deserialize, Clone, Debug)]
#[expect(dead_code)]
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

/// Sends a GET request to Mojang's authentication servers to verify a client's Minecraft account.
///
/// **Purpose:**
///
/// This function is used to ensure that a client connecting to the server has a valid, premium Minecraft account. It's a crucial step in preventing unauthorized access and maintaining server security.
///
/// **How it Works:**
///
/// 1. A client with a premium account sends a login request to the Mojang session server.
/// 2. Mojang's servers verify the client's credentials and add the player to the their Servers
/// 3. Now our server will send a Request to the Session servers and check if the Player has joined the Session Server .
///
/// **Note:** This process helps prevent unauthorized access to the server and ensures that only legitimate Minecraft accounts can connect.
pub async fn authenticate(
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    server: &Arc<Server>,
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

pub fn unpack_textures(property: Property, config: &TextureConfig) -> Result<(), TextureError> {
    let from64 = general_purpose::STANDARD
        .decode(property.value)
        .map_err(|e| TextureError::DecodeError(e.to_string()))?;
    let textures: ProfileTextures =
        serde_json::from_slice(&from64).map_err(|e| TextureError::JSONError(e.to_string()))?;
    for texture in textures.textures {
        let url =
            Url::parse(&texture.1.url).map_err(|e| TextureError::InvalidURL(e.to_string()))?;
        is_texture_url_valid(url, config)?
    }
    Ok(())
}

pub fn is_texture_url_valid(url: Url, config: &TextureConfig) -> Result<(), TextureError> {
    let scheme = url.scheme();
    if !config.allowed_url_schemes.contains(&scheme.to_string()) {
        return Err(TextureError::DisallowedUrlScheme(scheme.to_string()));
    }
    let domain = url.domain().unwrap_or("");
    if !config.allowed_url_domains.contains(&domain.to_string()) {
        return Err(TextureError::DisallowedUrlDomain(domain.to_string()));
    }
    Ok(())
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

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Invalid URL")]
    InvalidURL(String),
    #[error("Invalid URL scheme for player texture: {0}")]
    DisallowedUrlScheme(String),
    #[error("Invalid URL domain for player texture: {0}")]
    DisallowedUrlDomain(String),
    #[error("Failed to decode base64 player texture: {0}")]
    DecodeError(String),
    #[error("Failed to parse JSON from player texture: {0}")]
    JSONError(String),
}
