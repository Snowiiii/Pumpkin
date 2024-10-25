use serde::Deserialize;

#[derive(Deserialize)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}
