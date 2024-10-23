mod s_acknowledge_finish_config;
mod s_client_information;
mod s_known_packs;
mod s_plugin_message;

use num_derive::FromPrimitive;
pub use s_acknowledge_finish_config::*;
pub use s_client_information::*;
pub use s_known_packs::*;
pub use s_plugin_message::*;

/// DO NOT CHANGE ORDER
/// This Enum has the exact order like vanilla, Vanilla parses their Packet IDs from the enum order. Its also way easier to port.
#[derive(FromPrimitive)]
pub enum ServerboundConfigPackets {
    ClientInformation,
    CookieResponse,
    PluginMessage,
    AcknowledgedFinish,
    KeepAlive,
    Pong,
    ResourcePackResponse,
    KnownPacks,
}
