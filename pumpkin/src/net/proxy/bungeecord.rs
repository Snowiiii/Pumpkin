use std::{net::IpAddr, net::SocketAddr};

use pumpkin_protocol::Property;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::net::{offline_uuid, GameProfile};

#[derive(Error, Debug)]
pub enum BungeeCordError {
    #[error("Failed to parse Address")]
    FailedParseAddress,
    #[error("Failed to parse UUID")]
    FailedParseUUID,
    #[error("Failed to parse Properties")]
    FailedParseProperties,
    #[error("Failed to make offline UUID")]
    FailedMakeOfflineUUID,
}

/// Attempts to login a player via `BungeeCord`.
///
/// This function should be called when receiving the `SLoginStart` packet.
/// It utilizes the `server_address` received in the `SHandShake` packet,
/// which may contain optional data about the client:
///
/// 1. IP address (if `ip_forward` is enabled on the `BungeeCord` server)
/// 2. UUID (if `ip_forward` is enabled on the `BungeeCord` server)
/// 3. Game profile properties (if `ip_forward` and `online_mode` are enabled on the `BungeeCord` server)
///
/// If any of the optional data is missing, the function will attempt to
/// determine the player's information locally.
pub async fn bungeecord_login(
    client_address: &Mutex<SocketAddr>,
    server_address: &str,
    name: String,
) -> Result<(IpAddr, GameProfile), BungeeCordError> {
    let data = server_address.split('\0').take(4).collect::<Vec<_>>();

    // Ip of player, only given if ip_forward on bungee is true
    let ip = match data.get(1) {
        Some(ip) => ip
            .parse()
            .map_err(|_| BungeeCordError::FailedParseAddress)?,
        None => client_address.lock().await.ip(),
    };

    // Uuid of player, only given if ip_forward on bungee is true
    let id = match data.get(2) {
        Some(uuid) => uuid.parse().map_err(|_| BungeeCordError::FailedParseUUID)?,
        None => offline_uuid(name.as_str()).map_err(|_| BungeeCordError::FailedMakeOfflineUUID)?,
    };

    // Read properties and get textures
    // Properties of player's game profile, only given if ip_forward and online_mode
    // on bungee both are true
    let properties: Vec<Property> = match data.get(3) {
        Some(properties) => {
            serde_json::from_str(properties).map_err(|_| BungeeCordError::FailedParseProperties)?
        }
        None => vec![],
    };

    Ok((
        ip,
        GameProfile {
            id,
            name,
            properties,
            profile_actions: None,
        },
    ))
}
