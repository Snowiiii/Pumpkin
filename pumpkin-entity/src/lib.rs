use entity_type::EntityType;

pub mod entity_type;

pub type EntityId = i32;

pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
}

impl Entity {
    pub fn new(entity_id: EntityId, entity_type: EntityType) -> Self {
        Self {
            entity_id,
            entity_type,
        }
    }
}
