use pumpkin_macros::client_packet;

#[derive(serde::Serialize)]
#[client_packet("configuration:disconnect")]
pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}
