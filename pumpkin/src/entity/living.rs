use std::sync::atomic::AtomicI32;

use crossbeam::atomic::AtomicCell;
use pumpkin_protocol::client::play::{CEntityStatus, CSetEntityMetadata, Metadata};

use super::Entity;

/// Represents a Living Entity (e.g. Player, Zombie, Enderman...)
pub struct LivingEntity {
    pub entity: Entity,
    pub time_until_regen: AtomicI32,
    pub last_damage_taken: AtomicCell<f32>,
    /// The entity's current health level.
    pub health: AtomicCell<f32>,
}

impl LivingEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            time_until_regen: AtomicI32::new(0),
            last_damage_taken: AtomicCell::new(0.0),
            health: AtomicCell::new(20.0),
        }
    }

    pub fn tick(&self) {
        if self
            .time_until_regen
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0
        {
            self.time_until_regen
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub async fn set_health(&self, health: f32) {
        self.health.store(health);
        // tell everyone entities health changed
        self.entity
            .world
            .broadcast_packet_all(&CSetEntityMetadata::new(
                self.entity.entity_id.into(),
                Metadata::new(9, 3.into(), health),
            ))
            .await;
    }

    /// Returns if the entity was damaged or not
    pub fn damage(&self, amount: f32) -> bool {
        let regen = self
            .time_until_regen
            .load(std::sync::atomic::Ordering::Relaxed);
        let last_damage = self.last_damage_taken.load();
        // TODO: check if bypasses iframe
        if regen > 10 {
            if amount <= last_damage {
                return false;
            }
        } else {
            self.time_until_regen
                .store(20, std::sync::atomic::Ordering::Relaxed);
        }

        self.last_damage_taken.store(amount);
        amount > 0.0
    }

    /// Kills the Entity
    ///
    /// This is similar to `kill` but Spawn Particles, Animation and plays death sound
    pub async fn kill(&self) {
        // Spawns death smoke particles
        self.entity
            .world
            .broadcast_packet_all(&CEntityStatus::new(self.entity.entity_id, 60))
            .await;
        // Plays the death sound and death animation
        self.entity
            .world
            .broadcast_packet_all(&CEntityStatus::new(self.entity.entity_id, 3))
            .await;
        self.entity.remove().await;
    }
}
