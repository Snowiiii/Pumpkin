use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:pick_item_from_block")]
pub struct SPickItemFromBlock {
    pub pos: WorldPosition,
    pub include_data: bool,
}

// TODO: Handler for this packet
#[derive(Deserialize)]
#[server_packet("play:pick_item_from_entity")]
pub struct SPickItemFromEntity {
    pub id: i32,
    pub include_data: bool,
}
