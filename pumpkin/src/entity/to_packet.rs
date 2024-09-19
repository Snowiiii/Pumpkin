use crate::entity::Entity;
use pumpkin_protocol::client::play::CSpawnEntity;

pub fn spawn_entity_with_data(entity: &Entity, data: Option<i32>) -> CSpawnEntity {
    let pos = entity.pos.load();
    let velocity = entity.velocity.load();
    CSpawnEntity::new(
        entity.entity_id.into(),
        entity.uuid,
        (entity.entity_type as i32).into(),
        pos.x,
        pos.y,
        pos.z,
        entity.pitch.load(),
        entity.yaw.load(),
        entity.head_yaw.load(),
        data.unwrap_or(0).into(),
        velocity.x,
        velocity.y,
        velocity.z,
    )
}

impl From<&Entity> for CSpawnEntity {
    fn from(entity: &Entity) -> Self {
        spawn_entity_with_data(entity, None)
    }
}
