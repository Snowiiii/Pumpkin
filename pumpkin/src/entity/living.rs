use crossbeam::atomic::AtomicCell;
use pumpkin_protocol::client::play::{CEntityStatus, CSetEntityMetadata, Metadata};

use super::Entity;

/// Represents a Living Entity (e.g. Player, Zombie, Enderman...)
pub struct LivingEntity {
    pub entity: Entity,
    /// The entity's current health level.
    pub health: AtomicCell<f32>,
}

impl LivingEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            health: AtomicCell::new(20.0),
        }
    }

    pub fn set_health(&self, health: f32) {
        self.health.store(health);
        // tell everyone entities health changed
        self.entity
            .world
            .broadcast_packet_all(&CSetEntityMetadata::new(
                self.entity.entity_id.into(),
                Metadata::new(9, 3.into(), health),
            ));
    }

    /// Kills the Entity
    ///
    /// This is similar to `kill` but Spawn Particles, Animation and plays death sound
    pub fn kill(&self) {
        // Spawns death smoke particles
        self.entity
            .world
            .broadcast_packet_all(&CEntityStatus::new(self.entity.entity_id, 60));
        // Plays the death sound and death animation
        self.entity
            .world
            .broadcast_packet_all(&CEntityStatus::new(self.entity.entity_id, 3));
        self.entity.remove();
    }
}
