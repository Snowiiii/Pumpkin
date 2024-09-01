use entity_type::EntityType;
use pose::EntityPose;
use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
};

pub mod entity_type;
pub mod pose;

pub type EntityId = i32;

pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub pos: Vector3<f64>,
    pub block_pos: WorldPosition,
    pub chunk_pos: Vector2<i32>,

    pub yaw: f32,
    pub head_yaw: f32,
    pub pitch: f32,
    // TODO: Change this in diffrent poses
    pub standing_eye_height: f32,
    pub pose: EntityPose,
}

impl Entity {
    pub fn new(entity_id: EntityId, entity_type: EntityType, standing_eye_height: f32) -> Self {
        Self {
            entity_id,
            entity_type,
            pos: Vector3::new(0.0, 0.0, 0.0),
            block_pos: WorldPosition(Vector3::new(0, 0, 0)),
            chunk_pos: Vector2::new(0, 0),
            yaw: 0.0,
            head_yaw: 0.0,
            pitch: 0.0,
            standing_eye_height,
            pose: EntityPose::Standing,
        }
    }

    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        if self.pos.x != x || self.pos.y != y || self.pos.z != z {
            self.pos = Vector3::new(x, y, z);
            let i = x.floor() as i32;
            let j = y.floor() as i32;
            let k = z.floor() as i32;

            let block_pos = self.block_pos.0;
            if i != block_pos.x || j != block_pos.y || k != block_pos.z {
                self.block_pos = WorldPosition(Vector3::new(i, j, k));

                if get_section_cord(i) != self.chunk_pos.x
                    || get_section_cord(k) != self.chunk_pos.z
                {
                    self.chunk_pos =
                        Vector2::new(get_section_cord(block_pos.x), get_section_cord(block_pos.z));
                }
            }
        }
    }
}
