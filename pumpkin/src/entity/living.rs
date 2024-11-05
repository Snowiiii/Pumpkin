use std::sync::atomic::AtomicI32;

use crossbeam::atomic::AtomicCell;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_protocol::client::play::{CDamageEvent, CEntityStatus, CSetEntityMetadata, Metadata};

use super::Entity;

/// Represents a living entity within the game world.
///
/// This struct encapsulates the core properties and behaviors of living entities, including players, mobs, and other creatures.
pub struct LivingEntity {
    /// The underlying entity object, providing basic entity information and functionality.
    pub entity: Entity,
    /// Previously last known position of the entity
    pub last_pos: AtomicCell<Vector3<f64>>,
    /// Tracks the remaining time until the entity can regenerate health.
    pub time_until_regen: AtomicI32,
    /// Stores the amount of damage the entity last received.
    pub last_damage_taken: AtomicCell<f32>,
    /// The current health level of the entity.
    pub health: AtomicCell<f32>,
    /// The distance the entity has been falling
    pub fall_distance: AtomicCell<f64>,
}

impl LivingEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            last_pos: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            time_until_regen: AtomicI32::new(0),
            last_damage_taken: AtomicCell::new(0.0),
            health: AtomicCell::new(20.0),
            fall_distance: AtomicCell::new(0.0),
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

    pub fn set_pos(&self, x: f64, y: f64, z: f64) {
        self.last_pos.store(self.entity.pos.load());
        self.entity.set_pos(x, y, z);
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

    pub async fn damage(&self, amount: f32) {
        self.entity
            .world
            .broadcast_packet_all(&CDamageEvent::new(
                self.entity.entity_id.into(),
                // TODO add damage_type id
                0.into(),
                None,
                None,
                None,
            ))
            .await;

        let new_health = (self.health.load() - amount).max(0.0);
        self.set_health(new_health).await;

        if new_health == 0.0 {
            self.kill().await;
        }
    }

    /// Returns if the entity was damaged or not
    pub fn check_damage(&self, amount: f32) -> bool {
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

    pub async fn update_fall_distance(&self, dont_damage: bool) {
        let y = self.entity.pos.load().y;
        let last_y = self.last_pos.load().y;
        let grounded = self
            .entity
            .on_ground
            .load(std::sync::atomic::Ordering::Relaxed);

        // + => falling, - => up
        let y_diff = last_y - y;

        if grounded {
            let fall_distance = self.fall_distance.swap(0.0);
            if dont_damage {
                return;
            }

            let mut damage = (fall_distance - 3.0).max(0.0) as f32;
            damage = (damage * 2.0).round() / 2.0;
            if !self.check_damage(damage) {
                return;
            }

            self.damage(damage).await;
        } else if y_diff < 0.0 {
            self.fall_distance.store(0.0);
        } else {
            let fall_distance = self.fall_distance.load();
            self.fall_distance.store(fall_distance + y_diff);
        }
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
    }
}
