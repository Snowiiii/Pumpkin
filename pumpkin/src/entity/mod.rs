pub mod player;

pub type EntityId = i32;

pub struct Entity {
    pub entity_id: EntityId,
}

impl Entity {
    pub fn new(entity_id: EntityId) -> Self {
        Self { entity_id }
    }
}
