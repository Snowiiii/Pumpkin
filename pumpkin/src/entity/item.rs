use crate::entity::player::Player;
use crate::entity::{random_float, Entity};
use crate::server::Server;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_entity::entity_type::EntityType;
use pumpkin_entity::pose::EntityPose;
use pumpkin_entity::EntityId;
use pumpkin_inventory::Container;
use pumpkin_protocol::client::play::{CPickupItem, CSetEntityMetadata, CSpawnEntity, Metadata};
use pumpkin_protocol::slot::Slot;
use pumpkin_world::item::ItemStack;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct ItemEntity {
    pub item_stack: ItemStack,
    is_able_to_be_picked_up: Arc<AtomicBool>,
    pub entity: Arc<Entity>,
}

impl ItemEntity {
    pub async fn spawn(
        pos: Vector3<f64>,
        velocity: Vector3<f64>,
        world: Arc<World>,
        item_stack: ItemStack,
        server: Arc<Server>,
    ) {
        let is_able_to_be_picked_up = Arc::new(AtomicBool::new(false));
        {
            let is_able_to_be_picked_up = is_able_to_be_picked_up.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                is_able_to_be_picked_up.store(true, std::sync::atomic::Ordering::Relaxed);
            });
        }

        let entity = Arc::new(Entity::new(
            server.new_entity_id(),
            Uuid::new_v4(),
            world,
            EntityType::Item,
            0.,
        ));
        entity.velocity.store(velocity);
        entity.set_pos(pos.x, pos.y, pos.z);

        let item_entity = Self {
            item_stack,
            is_able_to_be_picked_up,
            entity: entity.clone(),
        };

        server
            .broadcast_packet_all(&entity.get_spawn_entity_packet(None))
            .await;
        {
            let server = server.clone();
            tokio::spawn(async move { item_entity.drop_loop(server).await });
        }
        let metadata = Metadata::new(8, 7.into(), Slot::from(&item_stack));
        server
            .broadcast_packet_all(&CSetEntityMetadata::new(entity.entity_id.into(), metadata))
            .await;
    }
    pub async fn spawn_from_player(
        player_entity: &Entity,
        item_stack: ItemStack,
        server: Arc<Server>,
    ) {
        let player_pos = player_entity.pos.load();
        let pos = Vector3 {
            x: player_pos.x,
            y: player_pos.y + f64::from(player_entity.standing_eye_height) - 0.3,
            z: player_pos.z,
        };
        Self::spawn(
            pos,
            toss_velocity(player_entity),
            player_entity.world.clone(),
            item_stack,
            server,
        )
        .await;
    }

    pub(self) async fn check_pickup(self) -> PickupEvent {
        if !self.is_able_to_be_picked_up.load(Ordering::Relaxed) {
            return PickupEvent::NotPickedUp(self);
        }

        let pos = self.entity.pos.load();
        for player in self.entity.world.current_players.lock().await.values() {
            let player_pos = player.living_entity.entity.pos.load();
            if (pos.x - player_pos.x).abs() <= 1.
                && (pos.z - player_pos.z).abs() <= 1.
                && (pos.y - player_pos.y).abs() <= 1.
            {
                let inventory = player.inventory.lock().await;
                if inventory.all_combinable_slots().par_iter().any(|slot| {
                    match slot {
                        None => true,
                        // TODO: Add check for max stack size here
                        Some(slot) => {
                            slot.item_id == self.item_stack.item_id
                                && slot.item_count + self.item_stack.item_count <= 64
                        }
                    }
                }) {
                    drop(inventory);
                    return PickupEvent::PickedUp {
                        entity_id: self.entity.entity_id,
                        item: self.item_stack,
                        player: player.clone(),
                    };
                }
            }
        }
        PickupEvent::NotPickedUp(self)
    }

    async fn handle_pickup(
        entity_id: EntityId,
        player: Arc<Player>,
        dropped_item: ItemStack,
        server: Arc<Server>,
    ) {
        let mut inventory = player.inventory.lock().await;

        server
            .broadcast_packet_all(&CPickupItem::new_item(
                entity_id.into(),
                player.living_entity.entity.entity_id.into(),
                dropped_item.item_count,
            ))
            .await;
        let mut index = None;

        for (slot_index, slot) in inventory.hotbar_mut().into_iter().enumerate() {
            match slot {
                None => {
                    *slot = Some(dropped_item);
                    index = Some(slot_index + 27);
                    break;
                }
                Some(item_stack) => {
                    // TODO: Add max stack size check here
                    if item_stack.item_id == dropped_item.item_id
                        && item_stack.item_count + dropped_item.item_count <= 64
                    {
                        item_stack.item_count += dropped_item.item_count;
                        index = Some(slot_index + 27);
                        break;
                    }
                }
            }
        }
        if index.is_none() {
            for (slot_index, slot) in inventory.main_inventory_mut().into_iter().enumerate() {
                match slot {
                    None => {
                        *slot = Some(dropped_item);
                        index = Some(slot_index);
                        break;
                    }
                    Some(item_stack) => {
                        // TODO: Add max stack size check here
                        if item_stack.item_id == dropped_item.item_id
                            && item_stack.item_count + dropped_item.item_count <= 64
                        {
                            item_stack.item_count += dropped_item.item_count;
                            index = Some(slot_index);
                            break;
                        }
                    }
                }
            }
        }
        drop(inventory);
        player
            .send_single_slot_inventory_change(
                index.expect("It needs to have a valid slot for this path to get loaded"),
            )
            .await;
    }

    async fn drop_loop(mut self, server: Arc<Server>) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / 20));
        let mut ticks = 0;
        loop {
            interval.tick().await;
            {
                match self.check_pickup().await {
                    PickupEvent::PickedUp {
                        entity_id,
                        player,
                        item: dropped_item,
                    } => {
                        Self::handle_pickup(entity_id, player, dropped_item, server).await;
                        break;
                    }
                    PickupEvent::NotPickedUp(s) => {
                        self = s;
                    }
                }
            }
            let old_position = self.entity.pos.load();
            self.entity.apply_gravity();
            /*if self.entity.on_ground.load(Ordering::Relaxed) {
                let mut pos = self.entity.pos.load();
                pos.y = pos.y.ceil();
                let velocity = self.entity.velocity.load().multiply(1.,0.,1.);
                self.entity.velocity.store(velocity);
                self.entity.set_pos(pos.x,pos.y,pos.z);
                dbg!(self.entity.pos.load().y);
            }*/
            //self.entity.bounds_check().await;
            self.entity.collision_check(false).await;
            let on_ground = self.entity.on_ground.load(Ordering::Relaxed);
            if !on_ground
                || self.entity.velocity.load().horizontal_length_squared() > 1.0e-5
                || ticks % 4 == 0
            {
                self.entity.advance_position();
                self.entity.collision_check(true).await;
                let on_ground = self.entity.on_ground.load(Ordering::Relaxed);
                let slipperiness = 0.98 * if on_ground { 0.6 } else { 1. };

                let mut velocity =
                    self.entity
                        .velocity
                        .load()
                        .multiply(slipperiness, 0.98, slipperiness);
                if velocity.length_squared() < 1.0e-10 {
                    velocity.z = 0.;
                    velocity.y = 0.;
                    velocity.x = 0.;
                }
                if on_ground && velocity.y < 0. {
                    velocity = velocity.multiply(1., -0.5, 1.);
                }

                self.entity.velocity.store(velocity);
                self.entity.send_velocity(&server).await;
            }

            self.entity.send_position(old_position, &server).await;
            ticks += 1;
        }
    }
}

enum PickupEvent {
    PickedUp {
        entity_id: EntityId,
        item: ItemStack,
        player: Arc<Player>,
    },
    NotPickedUp(ItemEntity),
}

fn toss_velocity(player: &Entity) -> Vector3<f64> {
    use std::f64::consts::PI;
    let pitch_sin = f64::sin(f64::from(player.pitch.load()) * (PI / 180.0));
    let pitch_cos = f64::cos(f64::from(player.pitch.load()) * (PI / 180.0));
    let yaw_sin = f64::sin(f64::from(player.yaw.load()) * (PI / 180.0));
    let yaw_cos = f64::cos(f64::from(player.yaw.load()) * (PI / 180.0));
    let random_angle = random_float() * (2.0 * PI);
    let random_offset = 0.02 * random_float();

    Vector3 {
        x: (-yaw_sin * pitch_cos).mul_add(0.3, f64::cos(random_angle) * random_offset),
        y: (-pitch_sin).mul_add(0.3, (random_float() - random_float()).mul_add(0.1, 0.1)),
        z: (yaw_cos * pitch_cos).mul_add(0.3, f64::sin(random_angle) * random_offset),
    }
}
