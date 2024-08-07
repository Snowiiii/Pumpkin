use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x04)]
pub struct SChatCommand {
    pub command: String,
}
