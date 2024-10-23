mod c_ping_response;
mod c_status_response;

pub use c_ping_response::*;
pub use c_status_response::*;

/// DO NOT CHANGE ORDER
/// This Enum has the exact order like vanilla, Vanilla parses their Packet IDs from the enum order. Its also way easier to port.
#[repr(i32)]
pub enum ClientboundStatusPackets {
    StatusResponse,
    PingRequest,
}
