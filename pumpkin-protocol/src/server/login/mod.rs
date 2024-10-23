mod s_encryption_response;
mod s_login_response;
mod s_login_start;
mod s_plugin_response;

use num_derive::FromPrimitive;
pub use s_encryption_response::*;
pub use s_login_response::*;
pub use s_login_start::*;
pub use s_plugin_response::*;

#[derive(FromPrimitive)]
pub enum ServerboundLoginPackets {
    LoginStart,
    EncryptionResponse,
    PluginResponse,
    LoginAcknowledged,
    CookieResponse,
}
