use crate::world::World;
use crossbeam::atomic::AtomicCell;
use itertools::Itertools;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
};
use pumpkin_entity::{entity_type::EntityType, pose::EntityPose, EntityId};
use pumpkin_protocol::{
    client::play::{CSetEntityMetadata, Metadata},
    VarInt,
};
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::coordinates::ChunkRelativeBlockCoordinates;
use rand::Rng;
use std::ops::Mul;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use uuid::Uuid;

pub mod item;
pub mod living;
pub mod player;
mod to_packet;

/// Represents a not living Entity (e.g. Item, Egg, Snowball...)
pub struct Entity {
    /// A unique identifier for the entity
    pub entity_id: EntityId,
    pub uuid: Uuid,
    /// The type of entity (e.g., player, zombie, item)
    pub entity_type: EntityType,
    /// The world in which the entity exists.
    pub world: Arc<World>,
    /// The entity's current health level.

    /// The entity's current position in the world
    pub pos: AtomicCell<Vector3<f64>>,
    /// The entity's position rounded to the nearest block coordinates
    pub block_pos: AtomicCell<WorldPosition>,
    /// The chunk coordinates of the entity's current position
    pub chunk_pos: AtomicCell<Vector2<i32>>,

    /// Indicates whether the entity is sneaking
    pub sneaking: AtomicBool,
    /// Indicates whether the entity is sprinting
    pub sprinting: AtomicBool,
    /// Indicates whether the entity is flying due to a fall
    pub fall_flying: AtomicBool,

    /// The entity's current velocity vector, aka Knockback
    pub velocity: AtomicCell<Vector3<f64>>,

    /// Indicates whether the entity is on the ground (may not always be accurate).
    pub on_ground: AtomicBool,

    /// The entity's yaw rotation (horizontal rotation) ← →
    pub yaw: AtomicCell<f32>,
    /// The entity's head yaw rotation (horizontal rotation of the head)
    pub head_yaw: AtomicCell<f32>,
    /// The entity's pitch rotation (vertical rotation) ↑ ↓
    pub pitch: AtomicCell<f32>,
    /// The height of the entity's eyes from the ground.
    pub standing_eye_height: f32,
    /// The entity's current pose (e.g., standing, sitting, swimming).
    pub pose: AtomicCell<EntityPose>,
}

impl Entity {
    pub fn new(
        entity_id: EntityId,
        uuid: Uuid,
        world: Arc<World>,
        entity_type: EntityType,
        standing_eye_height: f32,
    ) -> Self {
        Self {
            entity_id,
            uuid,
            entity_type,
            on_ground: AtomicBool::new(false),
            pos: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            block_pos: AtomicCell::new(WorldPosition(Vector3::new(0, 0, 0))),
            chunk_pos: AtomicCell::new(Vector2::new(0, 0)),
            sneaking: AtomicBool::new(false),
            world,
            // TODO: Load this from previous instance
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: AtomicCell::new(0.0),
            head_yaw: AtomicCell::new(0.0),
            pitch: AtomicCell::new(0.0),
            velocity: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            standing_eye_height,
            pose: AtomicCell::new(EntityPose::Standing),
        }
    }

    /// Updates the entity's position, block position, and chunk position.
    ///
    /// This function calculates the new position, block position, and chunk position based on the provided coordinates. If any of these values change, the corresponding fields are updated.
    pub fn set_pos(&self, x: f64, y: f64, z: f64) {
        let pos = self.pos.load();
        if pos.x != x || pos.y != y || pos.z != z {
            self.pos.store(Vector3::new(x, y, z));
            let i = x.floor() as i32;
            let j = y.floor() as i32;
            let k = z.floor() as i32;

            let block_pos = self.block_pos.load();
            let block_pos_vec = block_pos.0;
            if i != block_pos_vec.x || j != block_pos_vec.y || k != block_pos_vec.z {
                let new_block_pos = Vector3::new(i, j, k);
                self.block_pos.store(WorldPosition(new_block_pos));

                let chunk_pos = self.chunk_pos.load();
                if get_section_cord(i) != chunk_pos.x || get_section_cord(k) != chunk_pos.z {
                    self.chunk_pos.store(Vector2::new(
                        get_section_cord(new_block_pos.x),
                        get_section_cord(new_block_pos.z),
                    ));
                }
            }
        }
    }

    /// Sets the Entity yaw & pitch Rotation
    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        // TODO
        self.yaw.store(yaw);
        self.pitch.store(pitch);
    }

    /// Removes the Entity from their current World
    pub async fn remove(&self) {
        self.world.remove_entity(self).await;
    }

    pub async fn advance_with_velocity(&self) {
        let pos = self.pos.load();
        self.collision_check().await;
        let velocity = self.velocity.load();
        self.bounds_check(velocity).await;
        if self.on_ground.load(Ordering::Relaxed) {
            return;
        }
        self.set_pos(pos.x + velocity.x, pos.y + velocity.y, pos.z + velocity.z);
    }

    async fn collision_check(&self) {
        // TODO: Collision check with other entities.
        let velocity = self.velocity.load();
        if velocity.length_squared() == 0. {
            return;
        }
        let mut old_pos = self.pos.load();
        let pos = old_pos.add(&velocity);
        let chunk_pos = Vector2::new(
            get_section_cord(pos.x as i32),
            get_section_cord(pos.z as i32),
        );

        let (sender, mut receiver) = tokio::sync::mpsc::channel(1024);

        let level = self.world.level.clone();
        tokio::task::spawn_blocking(move || level.lock().fetch_chunks(&[chunk_pos], sender, false));

        if let Some(Ok(result)) = receiver.recv().await {
            let block_id = result
                .blocks
                .get_block(ChunkRelativeBlockCoordinates::from(pos));
            // TODO: Add check for other blocks that affect collision, like water
            if block_id.is_air() {
                return;
            }
            old_pos.y = pos.y.ceil();
            self.pos.store(old_pos);
            let mut velocity = self.velocity.load();
            velocity.y = velocity.y.min(0.);

            self.on_ground.store(true, Ordering::Relaxed);
        }
    }

    /// This function is used to check if an ItemEntity or an EXP orb is inside a block, and if so, get it out.
    async fn bounds_check(&self, velocity: Vector3<f64>) {
        // TODO: use velocity to determine if collision *could* happen.
        // Using velocity means that we only have to look up the block below in case we will actually pass from one block_height to another.
        // Theoretically, this should decrease chunk-locking.
        // Might lead to increased lag when *many* items are on the ground. Might not. Need to benchmark.
        let pos = self.pos.load();
        let chunk_pos = self.chunk_pos.load();
        let block_pos = self.block_pos.load();

        let (sender, mut receiver) = tokio::sync::mpsc::channel(1024);
        {
            let level = self.world.level.clone();
            tokio::task::spawn_blocking(move || {
                level.lock().fetch_chunks(&[chunk_pos], sender, false)
            });
        }
        let mut chunks = vec![];
        if let Some(Ok(result)) = receiver.recv().await {
            let block_id = result
                .blocks
                .get_block(ChunkRelativeBlockCoordinates::from(block_pos));
            // TODO: Add check for other blocks that affect collision, like water
            if block_id.is_air() {
                return;
            }
            chunks.push(result);
        }
        // Ordered in North, South, West, East, Up
        let neighbours = [(0., -1.), (0., -1.), (-1., 0.), (1., 0.)]
            .into_iter()
            .map(|block_pos_change| {
                (
                    (block_pos_change.0 as i32, block_pos_change.1 as i32),
                    Vector3 {
                        x: pos.x + block_pos_change.1,
                        y: 0.,
                        z: pos.z + block_pos_change.0,
                    },
                )
            })
            .collect_vec();

        let chunks = self
            .get_chunks(chunks, neighbours.iter().map(|(_, pos)| *pos).collect_vec())
            .await;
        let mut new_direction = Vector3 {
            x: 0.,
            y: 1.,
            z: 0.,
        };
        if let Some(collisions) = self.get_all_collisions(neighbours, chunks) {
            let collision = collisions
                .first()
                .expect("If no collisions happen, function returns None");
            new_direction = Vector3 {
                x: collision.0 .1 as f64,
                y: 0.,
                z: collision.0 .0 as f64,
            };
        }
        let new_velocity = new_direction.mul(random_float() * 0.02);
        let velocity = velocity.mul(0.75).add(&new_velocity);
        self.velocity.store(velocity);
    }

    #[allow(clippy::type_complexity)]
    fn get_all_collisions(
        &self,
        neighbours: Vec<((i32, i32), Vector3<f64>)>,
        chunks: Vec<Arc<ChunkData>>,
    ) -> Option<Vec<((i32, i32), Vector3<f64>)>> {
        let mut collisions = vec![];
        for (block_pos_change, pos) in neighbours {
            let block = chunks
                .iter()
                .find_map(|chunk| {
                    if chunk.position.x == get_section_cord(pos.x as i32)
                        && chunk.position.z == get_section_cord(pos.z as i32)
                    {
                        Some(chunk.blocks.get_block(pos.into()))
                    } else {
                        None
                    }
                })
                .expect("All chunks should get loaded above");
            if !block.is_air() {
                continue;
            }
            collisions.push((block_pos_change, pos))
        }
        if collisions.is_empty() {
            None
        } else {
            Some(collisions)
        }
    }

    async fn get_chunks(
        &self,
        pre_existing_chunks: Vec<Arc<ChunkData>>,
        positions: Vec<Vector3<f64>>,
    ) -> Vec<Arc<ChunkData>> {
        let mut chunk_positions = vec![];
        for pos in positions {
            let chunk_pos = Vector2::new(
                get_section_cord(pos.x as i32),
                get_section_cord(pos.z as i32),
            );
            if !chunk_positions.contains(&chunk_pos)
                && !pre_existing_chunks
                    .iter()
                    .any(|chunk| chunk.position == chunk_pos)
            {
                chunk_positions.push(chunk_pos);
            }
        }
        let (sender, mut receiver) = tokio::sync::mpsc::channel(8096);
        {
            let level = self.world.level.clone();
            tokio::task::spawn_blocking(move || {
                level.lock().fetch_chunks(&chunk_positions, sender, true)
            });
        }
        let mut chunks = pre_existing_chunks;
        while let Some(Ok(chunk)) = receiver.recv().await {
            chunks.push(chunk)
        }
        chunks
    }

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    ///
    /// This function calculates the entity's new velocity based on the specified knockback strength and direction.
    pub fn knockback(&self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = self.velocity.load();
        self.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
    }

    pub async fn set_sneaking(&self, sneaking: bool) {
        assert!(self.sneaking.load(std::sync::atomic::Ordering::Relaxed) != sneaking);
        self.sneaking
            .store(sneaking, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Flag::Sneaking, sneaking).await;
        // if sneaking {
        //     self.set_pose(EntityPose::Crouching).await;
        // } else {
        //     self.set_pose(EntityPose::Standing).await;
        // }
    }

    pub async fn set_sprinting(&self, sprinting: bool) {
        assert!(self.sprinting.load(std::sync::atomic::Ordering::Relaxed) != sprinting);
        self.sprinting
            .store(sprinting, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Flag::Sprinting, sprinting).await;
    }

    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_fall_flying(&self, fall_flying: bool) {
        assert!(self.fall_flying.load(std::sync::atomic::Ordering::Relaxed) != fall_flying);
        self.fall_flying
            .store(fall_flying, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Flag::FallFlying, fall_flying).await;
    }

    async fn set_flag(&self, flag: Flag, value: bool) {
        let index = flag.to_u32().unwrap();
        let mut b = 0i8;
        if value {
            b |= 1 << index;
        } else {
            b &= !(1 << index);
        }
        let packet = CSetEntityMetadata::new(self.entity_id.into(), Metadata::new(0, 0.into(), b));
        self.world.broadcast_packet_all(&packet).await;
    }

    pub async fn set_pose(&self, pose: EntityPose) {
        self.pose.store(pose);
        let pose = pose as i32;
        let packet = CSetEntityMetadata::<VarInt>::new(
            self.entity_id.into(),
            Metadata::new(6, 20.into(), (pose).into()),
        );
        self.world.broadcast_packet_all(&packet).await;
    }

    // This gets run once per "tick" (tokio task sleeping to imitate tick)
    pub fn apply_gravity(&self) {
        if self.on_ground.load(Ordering::Relaxed) {
            return;
        }
        let mut velocity = self.velocity.load();
        velocity.y -= self.entity_type.gravity();
        self.velocity.store(velocity);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
/// Represents various entity flags that are sent in entity metadata.
///
/// These flags are used by the client to modify the rendering of entities based on their current state.
///
/// **Purpose:**
///
/// This enum provides a more type-safe and readable way to represent entity flags compared to using raw integer values.
pub enum Flag {
    /// Indicates if the entity is on fire.
    OnFire,
    /// Indicates if the entity is sneaking.
    Sneaking,
    /// Indicates if the entity is sprinting.
    Sprinting,
    /// Indicates if the entity is swimming.
    Swimming,
    /// Indicates if the entity is invisible.
    Invisible,
    /// Indicates if the entity is glowing.
    Glowing,
    /// Indicates if the entity is flying due to a fall.
    FallFlying,
}

pub fn random_float() -> f64 {
    rand::thread_rng().gen_range(0.0..=1.0)
}
