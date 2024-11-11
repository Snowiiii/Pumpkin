use pumpkin_macros::server_packet;

// Acknowledgement to the Login Success packet sent to the server.
#[derive(serde::Deserialize)]
#[server_packet("login:login_acknowledged")]
pub struct SLoginAcknowledged {}
