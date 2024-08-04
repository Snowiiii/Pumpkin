use std::net::IpAddr;

use num_bigint::BigInt;
use pumpkin_protocol::Property;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::server::Server;

#[derive(Deserialize, Clone, Debug)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<Property>,
}

pub fn authenticate(
    username: &str,
    server_hash: &str,
    ip: &IpAddr,
    server: &mut Server,
) -> Result<GameProfile, AuthError> {
    assert!(server.auth_client.is_some());
    let address = if server.base_config.prevent_proxy_connections {
        format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}&ip={ip}")
    } else {
        format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}")
    };
    let response = server
        .auth_client
        .as_ref()
        .unwrap()
        .get(address)
        .send()
        .map_err(|_| AuthError::FailedResponse)?;
    match response.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT => Err(AuthError::UnverifiedUsername)?,
        other => Err(AuthError::UnknownStatusCode(other.as_str().to_string()))?,
    }
    let profile: GameProfile = response.json().map_err(|_| AuthError::FailedParse)?;
    Ok(profile)
}

pub fn auth_digest(bytes: &[u8]) -> String {
    BigInt::from_signed_bytes_be(bytes).to_str_radix(16)
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication servers are down")]
    FailedResponse,
    #[error("Failed to verify username")]
    UnverifiedUsername,
    #[error("Failed to parse JSON into Game Profile")]
    FailedParse,
    #[error("Unknown Status Code")]
    UnknownStatusCode(String),
}
