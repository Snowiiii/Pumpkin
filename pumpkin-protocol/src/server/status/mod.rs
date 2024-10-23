mod s_ping_request;
mod s_status_request;

use num_derive::FromPrimitive;
pub use s_ping_request::*;
pub use s_status_request::*;

#[derive(FromPrimitive)]
pub enum ServerboundStatusPackets {
    StatusRequest,
    PingRequest,
}
