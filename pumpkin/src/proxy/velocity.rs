use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};
use hmac::{Hmac, Mac};
use pumpkin_config::proxy::VelocityConfig;
use pumpkin_protocol::{
    bytebuf::ByteBuffer, client::login::CLoginPluginRequest, server::login::SLoginPluginResponse,
};
use sha2::Sha256;

use crate::client::Client;

type HmacSha256 = Hmac<Sha256>;

const MAX_SUPPORTED_FORWARDING_VERSION: i32 = 4;
const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

pub fn velocity_login(client: &mut Client) {
    let velocity_message_id: i32 = 0;

    let mut buf = BytesMut::new();
    buf.put_u8(MAX_SUPPORTED_FORWARDING_VERSION as u8);
    client.send_packet(&CLoginPluginRequest::new(
        velocity_message_id.into(),
        PLAYER_INFO_CHANNEL,
        &buf,
    ));
}

pub fn check_integrity(data: (&[u8], &[u8]), secret: String) -> bool {
    let (signature, data_without_signature) = data;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data_without_signature);
    mac.verify_slice(signature).is_ok()
}

pub fn receive_plugin_response(
    client: &mut Client,
    config: VelocityConfig,
    response: SLoginPluginResponse,
) {
    dbg!("velocity response");
    if let Some(data) = response.data {
        let (signature, data_without_signature) = data.split_at(32);

        if !check_integrity((signature, data_without_signature), config.secret) {
            client.kick("Unable to verify player details");
            return;
        }
        let mut buf = ByteBuffer::new(BytesMut::new());
        buf.put_slice(data_without_signature);

        // check velocity version
        let version = buf.get_var_int();
        let version = version.0;
        if version > MAX_SUPPORTED_FORWARDING_VERSION {
            client.kick(&format!(
                "Unsupported forwarding version {version}, Max: {MAX_SUPPORTED_FORWARDING_VERSION}"
            ));
            return;
        }
        // TODO: no unwrap
        let addr: SocketAddr = buf.get_string().unwrap().parse().unwrap();
        client.address = addr;
        todo!()
    } else {
        client.kick("This server requires you to connect with Velocity.")
    }
}
