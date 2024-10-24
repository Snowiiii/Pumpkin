mod s_chat_command;
mod s_chat_message;
mod s_click_container;
mod s_client_information;
mod s_close_container;
mod s_confirm_teleport;
mod s_interact;
mod s_keep_alive;
mod s_ping_request;
mod s_player_abilities;
mod s_player_action;
mod s_player_command;
mod s_player_ground;
mod s_player_position;
mod s_player_position_rotation;
mod s_player_rotation;
mod s_set_creative_slot;
mod s_set_held_item;
mod s_swing_arm;
mod s_use_item;
mod s_use_item_on;

use num_derive::FromPrimitive;
pub use s_chat_command::*;
pub use s_chat_message::*;
pub use s_click_container::*;
pub use s_client_information::*;
pub use s_close_container::*;
pub use s_confirm_teleport::*;
pub use s_interact::*;
pub use s_keep_alive::*;
pub use s_ping_request::*;
pub use s_player_abilities::*;
pub use s_player_action::*;
pub use s_player_command::*;
pub use s_player_ground::*;
pub use s_player_position::*;
pub use s_player_position_rotation::*;
pub use s_player_rotation::*;
pub use s_set_creative_slot::*;
pub use s_set_held_item::*;
pub use s_swing_arm::*;
pub use s_use_item::*;
pub use s_use_item_on::*;

/// DO NOT CHANGE ORDER
/// This Enum has the exact order like vanilla, Vanilla parses their Packet IDs from the enum order. Its also way easier to port.
#[derive(FromPrimitive)]
pub enum ServerboundPlayPackets {
    TeleportConfirm,
    QueryBlockNbt,
    SelectBundleItem,
    SetDifficulty,
    ChatAck,
    ChatCommandUnsigned,
    ChatCommand,
    ChatMessage,
    ChatSessionUpdate,
    ChunkBatchAck,
    ClientStatus,
    ClientTickEnd,
    ClientSettings,
    TabComplete,
    ConfigurationAck,
    ClickWindowButton,
    ClickWindow,
    CloseWindow,
    SlotStateChange,
    CookieResponse,
    PluginMessage,
    DebugSampleSubscription,
    EditBook,
    QueryEntityNbt,
    InteractEntity,
    GenerateStructure,
    KeepAlive,
    LockDifficulty,
    PlayerPosition,
    PlayerPositionAndRotation,
    PlayerRotation,
    PlayerFlying,

    VehicleMove,
    SteerBoat,
    PickItem,
    DebugPing,
    CraftRecipeRequest,
    PlayerAbilities,
    PlayerDigging,
    EntityAction,
    PlayerInput,
    Pong,
    SetRecipeBookState,
    SetDisplayedRecipe,
    NameItem,
    ResourcePackStatus,
    AdvancementTab,
    SelectTrade,
    SetBeaconEffect,
    HeldItemChange,

    UpdateCommandBlock,
    UpdateCommandBlockMinecart,
    CreativeInventoryAction,
    UpdateJigsawBlock,
    UpdateStructureBlock,
    UpdateSign,

    Animation,
    Spectate,
    PlayerBlockPlacement,
    UseItem,
}
