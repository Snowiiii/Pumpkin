use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CLogin {
    entity_id: i32,
    is_hardcore: bool,
    dimension_names: Vec<String>,
    max_players: VarInt,
    view_distance: VarInt,
    simulated_distance: VarInt,
    reduced_debug_info: bool,
    enabled_respawn_screen: bool,
    limited_crafting: bool,
    dimension_type: VarInt,
    dimension_name: String,
    hashed_seed: i64,
    game_mode: u8,
    previous_gamemode: i8,
    debug: bool,
    is_flat: bool,
    has_death_loc: bool,
    death_dimension_name: Option<String>,
    death_loc: Option<i64>, // POSITION NOT STRING
    portal_cooldown: VarInt,
    enforce_secure_chat: bool,
}

impl CLogin {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_id: i32,
        is_hardcore: bool,
        dimension_names: Vec<String>,
        max_players: VarInt,
        view_distance: VarInt,
        simulated_distance: VarInt,
        reduced_debug_info: bool,
        enabled_respawn_screen: bool,
        limited_crafting: bool,
        dimension_type: VarInt,
        dimension_name: String,
        hashed_seed: i64,
        game_mode: u8,
        previous_gamemode: i8,
        debug: bool,
        is_flat: bool,
        has_death_loc: bool,
        death_dimension_name: Option<String>,
        death_loc: Option<i64>, // todo add block pos
        portal_cooldown: VarInt,
        enforce_secure_chat: bool,
    ) -> Self {
        Self {
            entity_id,
            is_hardcore,
            dimension_names,
            max_players,
            view_distance,
            simulated_distance,
            reduced_debug_info,
            enabled_respawn_screen,
            limited_crafting,
            dimension_type,
            dimension_name,
            hashed_seed,
            game_mode,
            previous_gamemode,
            debug,
            is_flat,
            has_death_loc,
            death_dimension_name,
            death_loc,
            portal_cooldown,
            enforce_secure_chat,
        }
    }
}

impl Packet for CLogin {
    const PACKET_ID: VarInt = 0x2B;
}

impl ClientPacket for CLogin {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i32(self.entity_id);
        bytebuf.put_bool(self.is_hardcore);
        bytebuf.put_list(&self.dimension_names, |buf, v| buf.put_string(v));
        bytebuf.put_var_int(self.max_players);
        bytebuf.put_var_int(self.view_distance);
        bytebuf.put_var_int(self.simulated_distance);
        bytebuf.put_bool(self.reduced_debug_info);
        bytebuf.put_bool(self.enabled_respawn_screen);
        bytebuf.put_bool(self.limited_crafting);
        bytebuf.put_var_int(self.dimension_type);
        bytebuf.put_string(&self.dimension_name);
        bytebuf.put_i64(self.hashed_seed);
        bytebuf.put_u8(self.game_mode);
        bytebuf.put_i8(self.previous_gamemode);
        bytebuf.put_bool(self.debug);
        bytebuf.put_bool(self.is_flat);
        bytebuf.put_bool(self.has_death_loc);
        if self.has_death_loc {
            bytebuf.put_string(self.death_dimension_name.as_ref().unwrap());
            bytebuf.put_i64(self.death_loc.unwrap());
        }
        bytebuf.put_var_int(self.portal_cooldown);
        bytebuf.put_bool(self.enforce_secure_chat);
    }
}
