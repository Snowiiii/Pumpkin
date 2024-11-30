use std::f32::consts::PI;

use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::{particle, sound};
use pumpkin_protocol::{
    client::play::{CEntityVelocity, CParticle},
    SoundCategory, VarInt,
};
use pumpkin_world::item::ItemStack;

use crate::{
    entity::{player::Player, Entity},
    world::World,
};

#[derive(Debug, Clone, Copy)]
pub enum AttackType {
    Knockback,
    Critical,
    Sweeping,
    Strong,
    Weak,
}

impl AttackType {
    pub async fn new(player: &Player, attack_cooldown_progress: f32) -> Self {
        let entity = &player.living_entity.entity;

        let sprinting = entity.sprinting.load(std::sync::atomic::Ordering::Relaxed);
        let on_ground = entity.on_ground.load(std::sync::atomic::Ordering::Relaxed);
        let sword = player
            .inventory
            .lock()
            .await
            .held_item()
            .is_some_and(ItemStack::is_sword);

        let is_strong = attack_cooldown_progress > 0.9;
        if sprinting && is_strong {
            return Self::Knockback;
        }

        // TODO: even more checks
        if is_strong && !on_ground {
            // !sprinting omitted
            return Self::Critical;
        }

        // TODO: movement speed check
        if sword && is_strong {
            // !is_crit, !is_knockback_hit, on_ground omitted
            return Self::Sweeping;
        }

        if is_strong {
            Self::Strong
        } else {
            Self::Weak
        }
    }
}

pub async fn handle_knockback(
    attacker_entity: &Entity,
    victim: &Player,
    victim_entity: &Entity,
    strength: f64,
) {
    let yaw = attacker_entity.yaw.load();

    let saved_velo = victim_entity.velocity.load();
    victim_entity.knockback(
        strength * 0.5,
        f64::from((yaw * (PI / 180.0)).sin()),
        f64::from(-(yaw * (PI / 180.0)).cos()),
    );

    let entity_id = VarInt(victim_entity.entity_id);
    let victim_velocity = victim_entity.velocity.load();

    let packet = &CEntityVelocity::new(
        &entity_id,
        victim_velocity.x as f32,
        victim_velocity.y as f32,
        victim_velocity.z as f32,
    );
    let velocity = attacker_entity.velocity.load();
    attacker_entity
        .velocity
        .store(velocity.multiply(0.6, 1.0, 0.6));

    victim_entity.velocity.store(saved_velo);
    victim.client.send_packet(packet).await;
}

pub async fn spawn_sweep_particle(attacker_entity: &Entity, world: &World, pos: &Vector3<f64>) {
    let yaw = attacker_entity.yaw.load();
    let d = -f64::from((yaw * (PI / 180.0)).sin());
    let e = f64::from((yaw * (PI / 180.0)).cos());

    let scale = 0.5;
    // TODO: use entity height
    let body_y = pos.y * 2.0 * scale;

    world
        .broadcast_packet_all(&CParticle::new(
            false,
            pos.x + d,
            body_y,
            pos.z + e,
            0.0,
            0.0,
            0.0,
            0.0,
            0,
            VarInt(i32::from(particle!("sweep_attack"))), // sweep
            &[],
        ))
        .await;
}

pub async fn player_attack_sound(pos: &Vector3<f64>, world: &World, attack_type: AttackType) {
    match attack_type {
        AttackType::Knockback => {
            world
                .play_sound(
                    sound!("entity.player.attack.knockback"),
                    SoundCategory::Players,
                    pos,
                )
                .await;
        }
        AttackType::Critical => {
            world
                .play_sound(
                    sound!("entity.player.attack.crit"),
                    SoundCategory::Players,
                    pos,
                )
                .await;
        }
        AttackType::Sweeping => {
            world
                .play_sound(
                    sound!("entity.player.attack.sweep"),
                    SoundCategory::Players,
                    pos,
                )
                .await;
        }
        AttackType::Strong => {
            world
                .play_sound(
                    sound!("entity.player.attack.strong"),
                    SoundCategory::Players,
                    pos,
                )
                .await;
        }
        AttackType::Weak => {
            world
                .play_sound(
                    sound!("entity.player.attack.weak"),
                    SoundCategory::Players,
                    pos,
                )
                .await;
        }
    };
}
