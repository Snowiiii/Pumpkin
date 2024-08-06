use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{ServerPacket, VarInt};

pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}

impl ServerPacket for SConfirmTeleport {
    const PACKET_ID: VarInt = 0x00;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            teleport_id: bytebuf.get_var_int(),
        }
    }
}

pub struct SChatCommand {
    pub command: String,
}

impl ServerPacket for SChatCommand {
    const PACKET_ID: VarInt = 0x04;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            command: bytebuf.get_string().unwrap(),
        }
    }
}

pub struct SPlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub ground: bool,
}

impl ServerPacket for SPlayerPosition {
    const PACKET_ID: VarInt = 0x1A;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            x: bytebuf.get_f64(),
            feet_y: bytebuf.get_f64(),
            z: bytebuf.get_f64(),
            ground: bytebuf.get_bool(),
        }
    }
}

pub struct SPlayerCommand {
    pub entitiy_id: VarInt,
    pub action: Action,
    pub jump_boost: VarInt,
}
#[derive(FromPrimitive)]
pub enum Action {
    StartSneaking = 0,
    StopSneaking,
    LeaveBed,
    StartSprinting,
    StopSprinting,
    StartHourseJump,
    OpenVehicleInventory,
    StartFlyingElytra,
}

impl ServerPacket for SPlayerCommand {
    const PACKET_ID: VarInt = 0x25;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            entitiy_id: bytebuf.get_var_int(),
            action: Action::from_i32(bytebuf.get_var_int()).unwrap(),
            jump_boost: bytebuf.get_var_int(),
        }
    }
}

pub struct SPlayerPositionRotation {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}

impl ServerPacket for SPlayerPositionRotation {
    const PACKET_ID: VarInt = 0x1B;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            x: bytebuf.get_f64(),
            feet_y: bytebuf.get_f64(),
            z: bytebuf.get_f64(),
            yaw: bytebuf.get_f32(),
            pitch: bytebuf.get_f32(),
            ground: bytebuf.get_bool(),
        }
    }
}

pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}

impl ServerPacket for SPlayerRotation {
    const PACKET_ID: VarInt = 0x1C;

    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Self {
        Self {
            yaw: bytebuf.get_f32(),
            pitch: bytebuf.get_f32(),
            ground: bytebuf.get_bool(),
        }
    }
}
