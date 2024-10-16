use std::net::{IpAddr, SocketAddr};

use bytes::{BufMut, BytesMut};
use hmac::{Hmac, Mac};
use pumpkin_config::proxy::VelocityConfig;
use pumpkin_protocol::{
    bytebuf::ByteBuffer,
    client::login::{CLoginPluginRequest, CLoginSuccess},
    server::login::SLoginPluginResponse,
    Property,
};
use sha2::Sha256;

use crate::client::{authentication::GameProfile, Client};

type HmacSha256 = Hmac<Sha256>;

const MAX_SUPPORTED_FORWARDING_VERSION: i32 = 4;
const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

pub fn velocity_login(client: &Client) {
    let velocity_message_id: i32 = 0;

    let mut buf = BytesMut::new();
    buf.put_u8(MAX_SUPPORTED_FORWARDING_VERSION as u8);
    client.send_packet(&CLoginPluginRequest::new(
        velocity_message_id.into(),
        PLAYER_INFO_CHANNEL,
        &buf,
    ));
}

pub fn check_integrity(data: (&[u8], &[u8]), secret: &str) -> bool {
    let (signature, data_without_signature) = data;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data_without_signature);
    mac.verify_slice(signature).is_ok()
}

pub fn receive_plugin_response(
    client: &Client,
    config: &VelocityConfig,
    response: SLoginPluginResponse,
) {
    dbg!("velocity response");
    if let Some(data) = response.data {
        let (signature, data_without_signature) = data.split_at(32);

        if !check_integrity((signature, data_without_signature), &config.secret) {
            client.kick("Unable to verify player details");
            return;
        }
        let mut buf = ByteBuffer::new(BytesMut::new());
        buf.put_slice(data_without_signature);

        // check velocity version
        let version = buf.get_var_int().unwrap();
        let version = version.0;
        if version > MAX_SUPPORTED_FORWARDING_VERSION {
            client.kick(&format!(
                "Unsupported forwarding version {version}, Max: {MAX_SUPPORTED_FORWARDING_VERSION}"
            ));
            return;
        }
        // TODO: no unwrap
        let addr: SocketAddr = SocketAddr::new(
            buf.get_string().unwrap().parse::<IpAddr>().unwrap(),
            client.address.lock().port(),
        );

        *client.address.lock() = addr;

        let uuid = buf.get_uuid().unwrap();

        let username = buf.get_string().unwrap();

        // Read game profile properties
        let properties = buf
            .get_list(|data| {
                let name = data.get_string()?;
                let value = data.get_string()?;
                let signature = data.get_option(|data| data.get_string())?;

                Ok(Property {
                    name,
                    value,
                    signature,
                })
            })
            .unwrap();

        client.send_packet(&CLoginSuccess {
            uuid: &uuid,
            username: &username,
            properties: &properties,
            strict_error_handling: false,
        });

        *client.gameprofile.lock() = Some(GameProfile {
            id: uuid,
            name: username,
            properties,
            profile_actions: None,
        });
    }
}
