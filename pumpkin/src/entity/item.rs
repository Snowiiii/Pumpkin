use crate::entity::Entity;
use crate::server::Server;
use crossbeam::atomic::AtomicCell;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_entity::entity_type::EntityType;
use pumpkin_entity::pose::EntityPose;
use pumpkin_protocol::client::play::{
    CSetEntityMetadata, CSpawnEntity, CUpdateEntityPos, Metadata,
};
use pumpkin_protocol::slot::Slot;
use pumpkin_world::item::ItemStack;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
pub struct ItemEntity {
    pub item_stack: ItemStack,
    _is_able_to_be_picked_up: Arc<AtomicBool>,
    pub entity: Arc<Entity>,
}

impl ItemEntity {
    pub fn new(player_entity: &Entity, item_stack: ItemStack, server: Arc<Server>) -> Self {
        let _is_able_to_be_picked_up = Arc::new(AtomicBool::new(false));
        let countdown = _is_able_to_be_picked_up.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            countdown.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        let player_pos = player_entity.pos.load();
        let pos = Vector3 {
            x: player_pos.x,
            y: player_pos.y + player_entity.standing_eye_height as f64 - 0.3,
            z: player_pos.z,
        };

        let entity = Arc::new(Entity {
            entity_id: server.new_entity_id(),
            uuid: Uuid::new_v4(),
            entity_type: EntityType::Item,
            world: player_entity.world.clone(),
            health: (0.).into(),
            pos: AtomicCell::new(pos),
            block_pos: player_entity.block_pos.load().into(),
            chunk_pos: player_entity.chunk_pos.load().into(),
            sneaking: false.into(),
            sprinting: false.into(),
            fall_flying: false.into(),
            velocity: toss_velocity(player_entity).into(),
            on_ground: false.into(),
            yaw: 0.0.into(),
            head_yaw: 0.0.into(),
            pitch: 0.0.into(),
            standing_eye_height: 0.0,
            pose: EntityPose::Standing.into(),
        });
        server.broadcast_packet_all(&CSpawnEntity::from(entity.as_ref()));
        {
            let entity = entity.clone();
            let server = server.clone();
            tokio::spawn(async move { drop_loop(entity, server).await });
        }
        server.broadcast_packet_all(&CSetEntityMetadata {
            entity_id: entity.entity_id.into(),
            metadata: Metadata::new(8, 7.into(), Slot::from(&item_stack)),
            end: 255,
        });
        Self {
            item_stack,
            _is_able_to_be_picked_up,
            entity,
        }
    }
}

async fn drop_loop(entity: Arc<Entity>, server: Arc<Server>) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / 20));
    loop {
        interval.tick().await;
        entity.apply_gravity();
        let pos_before = entity.pos.load();
        entity.advance_with_velocity();
        let mut velocity = entity.velocity.load();
        let slipperiness = 0.98;
        velocity.y *= 0.98;
        velocity.z *= slipperiness;
        velocity.x *= slipperiness;
        entity.velocity.store(velocity);
        let pos = entity.pos.load();
        let (dx, dy, dz) = (
            pos.x * 4096. - pos_before.x * 4096.,
            pos.y * 4096. - pos_before.y * 4096.,
            pos.z * 4096. - pos_before.z * 4096.,
        );
        server.broadcast_packet_all(&CUpdateEntityPos::new(
            entity.entity_id.into(),
            dx as i16,
            dy as i16,
            dz as i16,
            entity.on_ground.load(Ordering::Relaxed),
        ));
    }
}

fn toss_velocity(player: &Entity) -> Vector3<f64> {
    use std::f64::consts::PI;
    let pitch_sin = f64::sin(player.pitch.load() as f64 * (PI / 180.0));
    let pitch_cos = f64::cos(player.pitch.load() as f64 * (PI / 180.0));
    let yaw_sin = f64::sin(player.yaw.load() as f64 * (PI / 180.0));
    let yaw_cos = f64::cos(player.yaw.load() as f64 * (PI / 180.0));
    let random_angle = random_float() * (2.0 * PI);
    let random_offset = 0.02 * random_float();

    Vector3 {
        x: (-yaw_sin * pitch_cos * 0.3) + f64::cos(random_angle) * random_offset,
        y: -pitch_sin * 0.3 + 0.1 + (random_float() - random_float()) * 0.1,
        z: (yaw_cos * pitch_cos * 0.3) + f64::sin(random_angle) * random_offset,
    }
}
