mod c_encryption_request;
mod c_login_disconnect;
mod c_login_success;
mod c_plugin_request;
mod c_set_compression;

pub use c_encryption_request::*;
pub use c_login_disconnect::*;
pub use c_login_success::*;
pub use c_plugin_request::*;
pub use c_set_compression::*;

/// DO NOT CHANGE ORDER
/// This Enum has the exact order like vanilla, Vanilla parses their Packet IDs from the enum order. Its also way easier to port.
#[repr(i32)]
pub enum ClientboundLoginPackets {
    Disconnect,
    EncryptionRequest,
    LoginSuccess,
    SetCompression,
    LoginPluginRequest,
    CookieRequest,
}
