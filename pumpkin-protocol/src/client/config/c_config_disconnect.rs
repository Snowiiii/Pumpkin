use crate::bytebuf::packet_id::Packet;

#[derive(serde::Serialize)]
pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> Packet for CConfigDisconnect<'a> {
    const PACKET_ID: i32 = 0x02;
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}
