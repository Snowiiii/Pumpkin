use std::sync::Arc;

use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
};
use pumpkin_entity::{entity_type::EntityType, pose::EntityPose, EntityId};
use pumpkin_protocol::{
    client::play::{CSetEntityMetadata, Metadata},
    VarInt,
};

use crate::{client::Client, world::World};

pub mod player;

pub struct Entity {
    pub entity_id: EntityId,
    pub entity_type: EntityType,
    pub pos: Vector3<f64>,
    pub block_pos: WorldPosition,
    pub chunk_pos: Vector2<i32>,

    pub sneaking: bool,
    pub sprinting: bool,
    pub fall_flying: bool,
    pub velocity: Vector3<f64>,

    // Should be not trusted
    pub on_ground: bool,

    pub yaw: f32,
    pub head_yaw: f32,
    pub pitch: f32,
    // TODO: Change this in diffrent poses
    pub standing_eye_height: f32,
    pub pose: EntityPose,
}

// TODO: Remove client: &mut Client, world: Arc<tokio::sync::Mutex<World>> bs
impl Entity {
    pub fn new(entity_id: EntityId, entity_type: EntityType, standing_eye_height: f32) -> Self {
        Self {
            entity_id,
            entity_type,
            on_ground: false,
            pos: Vector3::new(0.0, 0.0, 0.0),
            block_pos: WorldPosition(Vector3::new(0, 0, 0)),
            chunk_pos: Vector2::new(0, 0),
            sneaking: false,
            sprinting: false,
            fall_flying: false,
            yaw: 0.0,
            head_yaw: 0.0,
            pitch: 0.0,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            standing_eye_height,
            pose: EntityPose::Standing,
        }
    }

    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        if self.pos.x != x || self.pos.y != y || self.pos.z != z {
            self.pos = Vector3::new(x, y, z);
            let i = x.floor() as i32;
            let j = y.floor() as i32;
            let k = z.floor() as i32;

            let block_pos = self.block_pos.0;
            if i != block_pos.x || j != block_pos.y || k != block_pos.z {
                self.block_pos = WorldPosition(Vector3::new(i, j, k));

                if get_section_cord(i) != self.chunk_pos.x
                    || get_section_cord(k) != self.chunk_pos.z
                {
                    self.chunk_pos =
                        Vector2::new(get_section_cord(block_pos.x), get_section_cord(block_pos.z));
                }
            }
        }
    }

    pub fn knockback(&mut self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x * x + z * z < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let var7 = self.velocity;
        self.velocity = Vector3::new(
            var7.x / 2.0 - var8.x,
            if self.on_ground {
                (var7.y / 2.0 + strength).min(0.4)
            } else {
                var7.y
            },
            var7.z / 2.0 - var8.z,
        );
    }

    pub async fn set_sneaking(
        &mut self,
        client: &mut Client,
        world: Arc<tokio::sync::Mutex<World>>,
        sneaking: bool,
    ) {
        assert!(self.sneaking != sneaking);
        self.sneaking = sneaking;
        self.set_flag(client, world, Self::SNEAKING_FLAG_INDEX, sneaking)
            .await;
        // if sneaking {
        //     self.set_pose(EntityPose::Crouching).await;
        // } else {
        //     self.set_pose(EntityPose::Standing).await;
        // }
    }

    pub async fn set_sprinting(
        &mut self,
        client: &mut Client,
        world: Arc<tokio::sync::Mutex<World>>,
        sprinting: bool,
    ) {
        assert!(self.sprinting != sprinting);
        self.sprinting = sprinting;
        self.set_flag(client, world, Self::SPRINTING_FLAG_INDEX, sprinting)
            .await;
    }

    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground
    }

    pub async fn set_fall_flying(
        &mut self,
        client: &mut Client,
        world: Arc<tokio::sync::Mutex<World>>,
        fall_flying: bool,
    ) {
        assert!(self.fall_flying != fall_flying);
        self.fall_flying = fall_flying;
        self.set_flag(client, world, Self::FALL_FLYING_FLAG_INDEX, fall_flying)
            .await;
    }

    pub const ON_FIRE_FLAG_INDEX: u32 = 0;
    pub const SNEAKING_FLAG_INDEX: u32 = 1;
    pub const SPRINTING_FLAG_INDEX: u32 = 3;
    pub const SWIMMING_FLAG_INDEX: u32 = 4;
    pub const INVISIBLE_FLAG_INDEX: u32 = 5;
    pub const GLOWING_FLAG_INDEX: u32 = 6;
    pub const FALL_FLYING_FLAG_INDEX: u32 = 7;
    async fn set_flag(
        &mut self,
        client: &mut Client,
        world: Arc<tokio::sync::Mutex<World>>,
        index: u32,
        value: bool,
    ) {
        let mut b = 0i8;
        if value {
            b |= 1 << index;
        } else {
            b &= !(1 << index);
        }
        let packet = CSetEntityMetadata::new(self.entity_id.into(), Metadata::new(0, 0.into(), b));
        client.send_packet(&packet);
        world
            .lock()
            .await
            .broadcast_packet(&[client.token], &packet);
    }

    pub async fn set_pose(
        &mut self,
        client: &mut Client,
        world: Arc<tokio::sync::Mutex<World>>,
        pose: EntityPose,
    ) {
        self.pose = pose;
        let pose = self.pose as i32;
        let packet = CSetEntityMetadata::<VarInt>::new(
            self.entity_id.into(),
            Metadata::new(6, 20.into(), (pose).into()),
        );
        client.send_packet(&packet);
        world
            .lock()
            .await
            .broadcast_packet(&[client.token], &packet)
    }
}
