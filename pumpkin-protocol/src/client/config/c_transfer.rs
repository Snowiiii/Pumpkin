use crate::VarInt;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("config:transfer")]
pub struct CTransfer<'a> {
    host: &'a str,
    port: &'a VarInt,
}

impl<'a> CTransfer<'a> {
    #[expect(dead_code)]
    pub fn new(host: &'a str, port: &'a VarInt) -> Self {
        Self { host, port }
    }
}
