use std::sync::{atomic::AtomicBool, Arc, Mutex};

use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
};
use pumpkin_entity::{entity_type::EntityType, pose::EntityPose, EntityId};
use pumpkin_protocol::{
    client::play::{CSetEntityMetadata, Metadata},
    VarInt,
};

use crate::world::World;

pub mod player;

pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub world: Arc<World>,

    pub pos: Mutex<Vector3<f64>>,
    pub block_pos: Mutex<WorldPosition>,
    pub chunk_pos: Mutex<Vector2<i32>>,

    pub sneaking: AtomicBool,
    pub sprinting: AtomicBool,
    pub fall_flying: AtomicBool,
    pub velocity: Mutex<Vector3<f64>>,

    // Should be not trusted
    pub on_ground: AtomicBool,

    pub yaw: Mutex<f32>,
    pub head_yaw: Mutex<f32>,
    pub pitch: Mutex<f32>,
    // TODO: Change this in diffrent poses
    pub standing_eye_height: f32,
    pub pose: Mutex<EntityPose>,
}

impl Entity {
    pub fn new(
        entity_id: EntityId,
        world: Arc<World>,
        entity_type: EntityType,
        standing_eye_height: f32,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            on_ground: AtomicBool::new(false),
            pos: Mutex::new(Vector3::new(0.0, 0.0, 0.0)),
            block_pos: Mutex::new(WorldPosition(Vector3::new(0, 0, 0))),
            chunk_pos: Mutex::new(Vector2::new(0, 0)),
            sneaking: AtomicBool::new(false),
            world,
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: Mutex::new(0.0),
            head_yaw: Mutex::new(0.0),
            pitch: Mutex::new(0.0),
            velocity: Mutex::new(Vector3::new(0.0, 0.0, 0.0)),
            standing_eye_height,
            pose: Mutex::new(EntityPose::Standing),
        }
    }

    pub fn set_pos(&self, x: f64, y: f64, z: f64) {
        let mut pos = self.pos.lock().unwrap();
        if pos.x != x || pos.y != y || pos.z != z {
            *pos = Vector3::new(x, y, z);
            let i = x.floor() as i32;
            let j = y.floor() as i32;
            let k = z.floor() as i32;

            let mut block_pos = self.block_pos.lock().unwrap();
            let block_pos_vec = block_pos.0;
            if i != block_pos_vec.x || j != block_pos_vec.y || k != block_pos_vec.z {
                *block_pos = WorldPosition(Vector3::new(i, j, k));

                let mut chunk_pos = self.chunk_pos.lock().unwrap();
                if get_section_cord(i) != chunk_pos.x || get_section_cord(k) != chunk_pos.z {
                    *chunk_pos = Vector2::new(
                        get_section_cord(block_pos_vec.x),
                        get_section_cord(block_pos_vec.z),
                    );
                }
            }
        }
    }

    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        // TODO
        *self.yaw.lock().unwrap() = yaw;
        *self.pitch.lock().unwrap() = pitch
    }

    pub async fn remove(&mut self) {
        self.world.remove_entity(self);
    }

    pub fn knockback(&self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x * x + z * z < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let mut velocity = self.velocity.lock().unwrap();
        *velocity = Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        );
    }

    pub async fn set_sneaking(&self, sneaking: bool) {
        assert!(self.sneaking.load(std::sync::atomic::Ordering::Relaxed) != sneaking);
        self.sneaking
            .store(sneaking, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Self::SNEAKING_FLAG_INDEX, sneaking).await;
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
        self.set_flag(Self::SPRINTING_FLAG_INDEX, sprinting).await;
    }

    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_fall_flying(&self, fall_flying: bool) {
        assert!(self.fall_flying.load(std::sync::atomic::Ordering::Relaxed) != fall_flying);
        self.fall_flying
            .store(fall_flying, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Self::FALL_FLYING_FLAG_INDEX, fall_flying)
            .await;
    }

    pub const ON_FIRE_FLAG_INDEX: u32 = 0;
    pub const SNEAKING_FLAG_INDEX: u32 = 1;
    pub const SPRINTING_FLAG_INDEX: u32 = 3;
    pub const SWIMMING_FLAG_INDEX: u32 = 4;
    pub const INVISIBLE_FLAG_INDEX: u32 = 5;
    pub const GLOWING_FLAG_INDEX: u32 = 6;
    pub const FALL_FLYING_FLAG_INDEX: u32 = 7;
    async fn set_flag(&self, index: u32, value: bool) {
        let mut b = 0i8;
        if value {
            b |= 1 << index;
        } else {
            b &= !(1 << index);
        }
        let packet = CSetEntityMetadata::new(self.entity_id.into(), Metadata::new(0, 0.into(), b));
        self.world.broadcast_packet_all(&packet);
    }

    pub async fn set_pose(&self, pose: EntityPose) {
        *self.pose.lock().unwrap() = pose;
        let pose = pose as i32;
        let packet = CSetEntityMetadata::<VarInt>::new(
            self.entity_id.into(),
            Metadata::new(6, 20.into(), (pose).into()),
        );
        self.world.broadcast_packet_all(&packet)
    }
}
