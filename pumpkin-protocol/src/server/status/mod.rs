mod s_ping_request;
mod s_status_request;

use num_derive::FromPrimitive;
pub use s_ping_request::*;
pub use s_status_request::*;

/// DO NOT CHANGE ORDER
/// This Enum has the exact order like vanilla, Vanilla parses their Packet IDs from the enum order. Its also way easier to port.
#[derive(FromPrimitive)]
pub enum ServerboundStatusPackets {
    StatusRequest,
    PingRequest,
}
