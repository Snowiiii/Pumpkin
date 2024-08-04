use crate::{bytebuf::ByteBuffer, text::Text, BitSet, ClientPacket, VarInt};

pub struct ChunkDataAndUpdateLight {
    //Not implemented yet
}

pub struct ChunkEntry {
    pub block_count: i16,
}

pub struct SetHeldItem {
    slot: i8,
}

impl SetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}

impl ClientPacket for SetHeldItem {
    const PACKET_ID: VarInt = 0x53;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i8(self.slot);
    }
}

pub struct CPlayerAbilities {
    flags: i8,
    flying_speed: f32,
    field_of_view: f32,
}

impl CPlayerAbilities {
    pub fn new(flags: i8, flying_speed: f32, field_of_view: f32) -> Self {
        Self {
            flags,
            flying_speed,
            field_of_view,
        }
    }
}

impl ClientPacket for CPlayerAbilities {
    const PACKET_ID: VarInt = 0x38;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i8(self.flags);
        bytebuf.put_f32(self.flying_speed);
        bytebuf.put_f32(self.field_of_view);
    }
}

pub struct CPlayDisconnect {
    reason: Text,
}

impl CPlayDisconnect {
    pub fn new(reason: Text) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CPlayDisconnect {
    const PACKET_ID: VarInt = 0x1D;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_slice(&self.reason.encode());
    }
}

pub struct CChangeDifficulty {
    difficulty: u8,
    locked: bool,
}

impl CChangeDifficulty {
    pub fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}

impl ClientPacket for CChangeDifficulty {
    const PACKET_ID: VarInt = 0x0B;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_u8(self.difficulty);
        bytebuf.put_bool(self.locked);
    }
}
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

impl ClientPacket for CLogin {
    const PACKET_ID: VarInt = 0x2B;

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

pub struct CGameEvent {
    event: u8,
    value: f32,
}

impl CGameEvent {
    pub fn new(event: u8, value: f32) -> Self {
        Self { event, value }
    }
}

impl ClientPacket for CGameEvent {
    const PACKET_ID: VarInt = 0x22;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_u8(self.event);
        bytebuf.put_f32(self.value);
    }
}

pub struct CSyncPlayerPostion {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    flags: i8,
    teleport_id: VarInt,
}

impl CSyncPlayerPostion {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: i8,
        teleport_id: VarInt,
    ) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags,
            teleport_id,
        }
    }
}

impl ClientPacket for CSyncPlayerPostion {
    const PACKET_ID: VarInt = 0x40;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_f64(self.x);
        bytebuf.put_f64(self.y);
        bytebuf.put_f64(self.z);
        bytebuf.put_f32(self.yaw.to_degrees());
        bytebuf.put_f32(self.pitch.to_degrees());
        bytebuf.put_i8(self.flags);
        bytebuf.put_var_int(self.teleport_id);
    }
}

pub struct CChunkDataUpdateLight {
    chunk_x: i32,
    chunk_y: i32,
    heightmaps: Vec<u8>,
    data: Vec<u8>,
    block_entites: Vec<BlockEntity>,
    sky_light_mask: BitSet,
    block_light_mask: BitSet,
    empty_sky_light_mask: BitSet,
    sky_lights: Vec<SkyLight>,
    block_lights: Vec<BlockLight>,
}

impl CChunkDataUpdateLight {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chunk_x: i32,
        chunk_y: i32,
        heightmaps: Vec<u8>,
        data: Vec<u8>,
        block_entites: Vec<BlockEntity>,
        sky_light_mask: BitSet,
        block_light_mask: BitSet,
        empty_sky_light_mask: BitSet,
        sky_lights: Vec<SkyLight>,
        block_lights: Vec<BlockLight>,
    ) -> Self {
        Self {
            chunk_x,
            chunk_y,
            heightmaps,
            data,
            block_entites,
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            sky_lights,
            block_lights,
        }
    }
}

impl ClientPacket for CChunkDataUpdateLight {
    const PACKET_ID: VarInt = 0x27;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i32(self.chunk_x);
        bytebuf.put_i32(self.chunk_y);
        bytebuf.put_slice(&self.heightmaps);
        bytebuf.put_var_int(self.data.len() as VarInt);
        bytebuf.put_slice(&self.data);
        bytebuf.put_list::<BlockEntity>(&self.block_entites, |p, v| {
            p.put_u8(v.packed_xz);
            p.put_i16(v.y);
            p.put_var_int(v.typee);
            p.put_slice(&v.data);
        });
        bytebuf.put_bit_set(&self.sky_light_mask);
        bytebuf.put_bit_set(&self.block_light_mask);
        bytebuf.put_bit_set(&self.empty_sky_light_mask);
        bytebuf.put_list::<SkyLight>(&self.sky_lights, |p, v| {
            p.put_var_int(v.array.len() as VarInt);
            p.put_slice(&v.array);
        });
        bytebuf.put_list::<BlockLight>(&self.block_lights, |p, v| {
            p.put_var_int(v.array.len() as VarInt);
            p.put_slice(&v.array);
        });
    }
}

pub struct BlockEntity {
    packed_xz: u8,
    y: i16,
    typee: VarInt,
    data: Vec<u8>,
}

pub struct SkyLight {
    pub array: Vec<u8>,
}

pub struct BlockLight {
    pub array: Vec<u8>,
}
