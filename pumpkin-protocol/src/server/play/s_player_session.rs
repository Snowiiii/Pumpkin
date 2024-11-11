use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:chat_session_update")]
pub struct SPlayerSession {
    #[serde(with = "uuid::serde::compact")]
    pub uuid: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}
