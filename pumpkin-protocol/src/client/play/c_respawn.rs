use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:respawn")]
pub struct CRespawn<'a> {
    dimension_type: VarInt,
    dimension_name: &'a str,
    hashed_seed: i64,
    game_mode: u8,
    previous_gamemode: i8,
    debug: bool,
    is_flat: bool,
    death_dimension_name: Option<(&'a str, WorldPosition)>,
    portal_cooldown: VarInt,
    sealevel: VarInt,
    data_kept: u8,
}

impl<'a> CRespawn<'a> {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        dimension_type: VarInt,
        dimension_name: &'a str,
        hashed_seed: i64,
        game_mode: u8,
        previous_gamemode: i8,
        debug: bool,
        is_flat: bool,
        death_dimension_name: Option<(&'a str, WorldPosition)>,
        portal_cooldown: VarInt,
        sealevel: VarInt,
        data_kept: u8,
    ) -> Self {
        Self {
            dimension_type,
            dimension_name,
            hashed_seed,
            game_mode,
            previous_gamemode,
            debug,
            is_flat,
            death_dimension_name,
            portal_cooldown,
            sealevel,
            data_kept,
        }
    }
}
