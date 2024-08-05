use pumpkin_macros::packet;

#[derive(serde::Serialize)]
#[packet(0x03)]
pub struct CFinishConfig {}

impl Default for CFinishConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CFinishConfig {
    pub fn new() -> Self {
        Self {}
    }
}
