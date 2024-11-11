use pumpkin_macros::client_packet;

#[derive(serde::Serialize)]
#[client_packet("configuration:finish_configuration")]
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
