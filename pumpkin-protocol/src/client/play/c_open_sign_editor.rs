use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:open_sign_editor")]
pub struct COpenSignEditor {
    location: WorldPosition,
    is_front_text: bool,
}

impl COpenSignEditor {
    pub fn new(location: WorldPosition, is_front_text: bool) -> Self {
        Self {
            location,
            is_front_text,
        }
    }
}
