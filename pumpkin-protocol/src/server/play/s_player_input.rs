use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:player_input")]
pub struct SPlayerInput {
    // Yep exactly how it looks like
    _input: i8,
}
