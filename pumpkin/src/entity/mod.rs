use std::sync::{atomic::AtomicBool, Arc};

use crossbeam::atomic::AtomicCell;
use num_derive::FromPrimitive;
use pumpkin_core::math::{
    boundingbox::{BoundingBox, BoundingBoxSize},
    get_section_cord,
    position::WorldPosition,
    vector2::Vector2,
    vector3::Vector3,
};
use pumpkin_entity::{entity_type::EntityType, pose::EntityPose, EntityId};
use pumpkin_protocol::{
    client::play::{CSetEntityMetadata, CTeleportEntity, Metadata},
    VarInt,
};

use crate::world::World;

pub mod living;
pub mod player;

/// Represents a not living Entity (e.g. Item, Egg, Snowball...)
pub struct Entity {
    /// A unique identifier for the entity
    pub entity_id: EntityId,
    /// The type of entity (e.g., player, zombie, item)
    pub entity_type: EntityType,
    /// The world in which the entity exists.
    pub world: Arc<World>,
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
    /// The bounding box of an entity (hitbox)
    pub bounding_box: AtomicCell<BoundingBox>,
    ///The size (width and height) of the bounding box
    pub bounding_box_size: AtomicCell<BoundingBoxSize>,
}

impl Entity {
    pub fn new(
        entity_id: EntityId,
        world: Arc<World>,
        entity_type: EntityType,
        standing_eye_height: f32,
        bounding_box: AtomicCell<BoundingBox>,
        bounding_box_size: AtomicCell<BoundingBoxSize>,
    ) -> Self {
        Self {
            entity_id,
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
            bounding_box,
            bounding_box_size,
        }
    }

    /// Updates the entity's position, block position, and chunk position.
    ///
    /// This function calculates the new position, block position, and chunk position based on the provided coordinates. If any of these values change, the corresponding fields are updated.
    pub fn set_pos(&self, new_position: Vector3<f64>) {
        let pos = self.pos.load();
        if pos != new_position {
            self.pos.store(new_position);
            self.bounding_box.store(BoundingBox::new_from_pos(
                pos.x,
                pos.y,
                pos.z,
                &self.bounding_box_size.load(),
            ));

            let floor_x = new_position.x.floor() as i32;
            let floor_y = new_position.y.floor() as i32;
            let floor_z = new_position.z.floor() as i32;

            let block_pos = self.block_pos.load();
            let block_pos_vec = block_pos.0;
            if floor_x != block_pos_vec.x
                || floor_y != block_pos_vec.y
                || floor_z != block_pos_vec.z
            {
                let new_block_pos = Vector3::new(floor_x, floor_y, floor_z);
                self.block_pos.store(WorldPosition(new_block_pos));

                let chunk_pos = self.chunk_pos.load();
                if get_section_cord(floor_x) != chunk_pos.x
                    || get_section_cord(floor_z) != chunk_pos.z
                {
                    self.chunk_pos.store(Vector2::new(
                        get_section_cord(new_block_pos.x),
                        get_section_cord(new_block_pos.z),
                    ));
                }
            }
        }
    }

    pub async fn teleport(&self, position: Vector3<f64>, yaw: f32, pitch: f32) {
        self.world
            .broadcast_packet_all(&CTeleportEntity::new(
                self.entity_id.into(),
                position,
                Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
                // TODO
                &[],
                self.on_ground.load(std::sync::atomic::Ordering::SeqCst),
            ))
            .await;
        self.set_pos(position);
        self.set_rotation(yaw, pitch);
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
        if sneaking {
            self.set_pose(EntityPose::Crouching).await;
        } else {
            self.set_pose(EntityPose::Standing).await;
        }
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
        let index = flag as u8;
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
            Metadata::new(6, 21.into(), pose.into()),
        );
        self.world.broadcast_packet_all(&packet).await;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
/// Represents various entity flags that are sent in entity metadata.
///
/// These flags are used by the client to modify the rendering of entities based on their current state.
///
/// **Purpose:**
///
/// This enum provides a more type-safe and readable way to represent entity flags compared to using raw integer values.
#[repr(u8)]
pub enum Flag {
    /// Indicates if the entity is on fire.
    OnFire = 0,
    /// Indicates if the entity is sneaking.
    Sneaking = 1,
    /// Indicates if the entity is sprinting.
    Sprinting = 3,
    /// Indicates if the entity is swimming.
    Swimming = 4,
    /// Indicates if the entity is invisible.
    Invisible = 5,
    /// Indicates if the entity is glowing.
    Glowing = 6,
    /// Indicates if the entity is flying due to a fall.
    FallFlying = 7,
}
