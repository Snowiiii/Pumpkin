use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x22)]
pub struct CGameEvent {
    event: u8,
    value: f32,
}

/// Somewhere you need to implement all the random stuff right?
impl CGameEvent {
    pub fn new(event: GameEvent, value: f32) -> Self {
        Self {
            event: event as u8,
            value,
        }
    }
}

#[repr(u8)]
pub enum GameEvent {
    NoRespawnBlockAvailable,
    BeginRaining,
    EndRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    ArrowHitPlayer,
    RainLevelChange,
    ThunderLevelChange,
    PlayPufferfishStringSound,
    PlayElderGuardianMobAppearance,
    EnabledRespawnScreen,
    LimitedCrafting,
    StartWaitingChunks,
}
