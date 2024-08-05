use pumpkin_macros::packet;

#[derive(serde::Serialize)]
#[packet(0x02)]
pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}
