use std::net::IpAddr;

use pumpkin_protocol::Property;
use thiserror::Error;

use crate::{
    client::authentication::{offline_uuid, GameProfile},
    Client,
};

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

pub fn bungeecord_login(
    client: &Client,
    username: String,
) -> Result<(IpAddr, GameProfile), BungeeCordError> {
    let server_address = client.server_address.lock();
    let data = server_address.split('\0').take(4).collect::<Vec<_>>();

    // Ip of player, only given if ip_forward on bungee is true
    let ip = match data.get(1) {
        Some(ip) => ip
            .parse()
            .map_err(|_| BungeeCordError::FailedParseAddress)?,
        None => client.address.lock().ip(),
    };

    // Uuid of player, only given if ip_forward on bungee is true
    let id = match data.get(2) {
        Some(uuid) => uuid.parse().map_err(|_| BungeeCordError::FailedParseUUID)?,
        None => {
            offline_uuid(username.as_str()).map_err(|_| BungeeCordError::FailedMakeOfflineUUID)?
        }
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
            name: username,
            properties,
            profile_actions: None,
        },
    ))
}
