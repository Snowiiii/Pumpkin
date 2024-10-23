mod c_acknowledge_block;
mod c_actionbar;
mod c_block_destroy_stage;
mod c_block_update;
mod c_center_chunk;
mod c_change_difficulty;
mod c_chunk_data;
mod c_close_container;
mod c_disguised_chat_message;
mod c_display_objective;
mod c_entity_animation;
mod c_entity_metadata;
mod c_entity_status;
mod c_entity_velocity;
mod c_game_event;
mod c_head_rot;
mod c_hurt_animation;
mod c_keep_alive;
mod c_login;
mod c_open_screen;
mod c_particle;
mod c_pickup_item;
mod c_ping_response;
mod c_play_disconnect;
mod c_player_abilities;
mod c_player_chat_message;
mod c_player_info_update;
mod c_player_remove;
mod c_remove_entities;
mod c_reset_score;
mod c_set_container_content;
mod c_set_container_property;
mod c_set_container_slot;
mod c_set_health;
mod c_set_held_item;
mod c_set_title;
mod c_spawn_entity;
mod c_subtitle;
mod c_sync_player_position;
mod c_system_chat_message;
mod c_teleport_entity;
mod c_unload_chunk;
mod c_update_entity_pos;
mod c_update_entity_pos_rot;
mod c_update_entity_rot;
mod c_update_objectives;
mod c_update_score;
mod c_worldevent;
mod player_action;

pub use c_acknowledge_block::*;
pub use c_actionbar::*;
pub use c_block_destroy_stage::*;
pub use c_block_update::*;
pub use c_center_chunk::*;
pub use c_change_difficulty::*;
pub use c_chunk_data::*;
pub use c_close_container::*;
pub use c_disguised_chat_message::*;
pub use c_display_objective::*;
pub use c_entity_animation::*;
pub use c_entity_metadata::*;
pub use c_entity_status::*;
pub use c_entity_velocity::*;
pub use c_game_event::*;
pub use c_head_rot::*;
pub use c_hurt_animation::*;
pub use c_keep_alive::*;
pub use c_login::*;
pub use c_open_screen::*;
pub use c_particle::*;
pub use c_pickup_item::*;
pub use c_ping_response::*;
pub use c_play_disconnect::*;
pub use c_player_abilities::*;
pub use c_player_chat_message::*;
pub use c_player_info_update::*;
pub use c_player_remove::*;
pub use c_remove_entities::*;
pub use c_reset_score::*;
pub use c_set_container_content::*;
pub use c_set_container_property::*;
pub use c_set_container_slot::*;
pub use c_set_health::*;
pub use c_set_held_item::*;
pub use c_set_title::*;
pub use c_spawn_entity::*;
pub use c_subtitle::*;
pub use c_sync_player_position::*;
pub use c_system_chat_message::*;
pub use c_teleport_entity::*;
pub use c_unload_chunk::*;
pub use c_update_entity_pos::*;
pub use c_update_entity_pos_rot::*;
pub use c_update_entity_rot::*;
pub use c_update_objectives::*;
pub use c_update_score::*;
pub use c_worldevent::*;
pub use player_action::*;

/// DO NOT CHANGE ORDER
/// This Enum has the exact order like vanilla, Vanilla parses their Packet IDs from the enum order. Its also way easier to port.
#[repr(i32)]
pub enum ClientboundPlayPackets {
    Bundle,
    SpawnEntity,
    SpawnExperienceOrb,
    EntityAnimation,
    Statistics,
    AcknowledgeBlockChanges,
    BlockBreakAnimation,
    BlockEntityData,
    BlockAction,
    BlockChange,
    BossBar,
    ServerDifficulty,
    ChunkBatchEnd,
    ChunkBatchBegin,
    ChunkBiomes,
    ClearTitles,
    TabComplete,
    DeclareCommands,
    CloseWindow,
    WindowItems,
    WindowProperty,
    SetSlot,
    CookieRequest,
    SetCooldown,
    CustomChatCompletions,
    PluginMessage,
    DamageEvent,
    DebugSample,
    DeleteChat,
    Disconnect,
    DisguisedChat,
    EntityStatus,
    EntityPositionSync,
    Explosion,
    UnloadChunk,
    ChangeGameState,
    OpenHorseWindow,
    HurtAnimation,
    InitializeWorldBorder,
    KeepAlive,
    ChunkData,
    Effect,
    Particle,
    UpdateLight,
    JoinGame,
    MapData,
    MerchantOffers,
    EntityRelativeMove,
    EntityRelativeMoveAndRotation,
    MoveMinecart,
    EntityRotation,
    VehicleMove,
    OpenBook,
    OpenWindow,
    OpenSignEditor,
    Ping,
    DebugPong,
    CraftRecipeResponse,
    PlayerAbilities,
    ChatMessage,
    EndCombatEvent,
    EnterCombatEvent,
    DeathCombatEvent,
    PlayerInfoRemove,
    PlayerInfoUpdate,
    FacePlayer,
    PlayerPositionAndLook,
    PlayerRotation,
    RecipeBookAdd,
    RecipeBookRemove,
    RecipeBookSettings,
    DestroyEntities,
    RemoveEntityEffect,
    ResetScore,
    ResourcePackRemove,
    ResourcePackSend,
    Respawn,
    EntityHeadLook,
    MultiBlockChange,
    SelectAdvancementsTab,
    ServerData,
    ActionBar,
    WorldBorderCenter,
    WorldBorderLerpSize,
    WorldBorderSize,
    WorldBorderWarningDelay,
    WorldBorderWarningReach,

    Camera,
    UpdateViewPosition,
    UpdateViewDistance,
    SetCursorItem,
    SpawnPosition,
    DisplayScoreboard,
    EntityMetadata,
    AttachEntity,
    EntityVelocity,
    EntityEquipment,
    SetExperience,
    UpdateHealth,
    HeldItemChange,
    ScoreboardObjective,
    SetPassengers,
    SetPlayerInventory,
    Teams,
    UpdateScore,
    UpdateSimulationDistance,
    SetTitleSubtitle,
    TimeUpdate,
    SetTitleText,
    SetTitleTimes,
    EntitySoundEffect,
    SoundEffect,

    ConfigurationStart,
    StopSound,
    StoreCookie,
    SystemChatMessage,
    PlayerListHeaderAndFooter,
    NBTQueryResponse,
    CollectItem,
    EntityTeleport,
    TickingState,
    TickingStep,
    Transfer,
    UpdateAdvancements,
    UpdateAttributes,
    EntityEffect,
    DeclareRecipes,
    Tags,
    ProjectilePower,
    CustomReportDetails,
    ServerLinks,
}

#[cfg(test)]
mod test {
    use crate::client::play::ClientboundPlayPackets;

    #[test]
    fn check() {
        assert_eq!(ClientboundPlayPackets::CollectItem as i32, 0x6F)
    }
}
