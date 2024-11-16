use pumpkin_macros::server_packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[server_packet("config:client_information")]
pub struct SClientInformationConfig {
    pub locale: String, // 16
    pub view_distance: i8,
    pub chat_mode: VarInt, // Varint
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: VarInt,
    pub text_filtering: bool,
    pub server_listing: bool,
}
