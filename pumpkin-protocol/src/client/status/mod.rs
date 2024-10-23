mod c_ping_response;
mod c_status_response;

pub use c_ping_response::*;
pub use c_status_response::*;

#[repr(i32)]
pub enum ClientboundStatusPackets {
    StatusResponse,
    PingRequest,
}
