use serde::Deserialize;

#[derive(Deserialize)]
pub struct SPlayPingRequest {
    pub payload: i64,
}
