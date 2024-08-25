use entity_type::EntityType;

pub mod entity_type;

pub type EntityId = i32;

pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub lastx: f64,
    pub lasty: f64,
    pub lastz: f64,
    pub yaw: f32,
    pub head_yaw: f32,
    pub pitch: f32,
    // TODO: Change this in diffrent poses
    pub standing_eye_height: f32,
}

impl Entity {
    pub fn new(entity_id: EntityId, entity_type: EntityType, standing_eye_height: f32) -> Self {
        Self {
            entity_id,
            entity_type,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            lastx: 0.0,
            lasty: 0.0,
            lastz: 0.0,
            yaw: 0.0,
            head_yaw: 0.0,
            pitch: 0.0,
            standing_eye_height,
        }
    }
}
