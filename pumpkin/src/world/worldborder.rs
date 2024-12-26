use pumpkin_protocol::client::play::{
    CInitializeWorldBorder, CSetBorderCenter, CSetBorderLerpSize, CSetBorderSize,
    CSetBorderWarningDelay, CSetBorderWarningDistance,
};

use crate::net::Client;

use super::World;

pub struct Worldborder {
    pub center_x: f64,
    pub center_z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
    pub portal_teleport_boundary: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub damage_per_block: f32,
    pub buffer: f32,
}

impl Worldborder {
    #[must_use]
    pub fn new(
        x: f64,
        z: f64,
        diameter: f64,
        speed: i64,
        warning_blocks: i32,
        warning_time: i32,
    ) -> Self {
        Self {
            center_x: x,
            center_z: z,
            old_diameter: diameter,
            new_diameter: diameter,
            speed,
            portal_teleport_boundary: 29_999_984,
            warning_blocks,
            warning_time,
            damage_per_block: 0.0,
            buffer: 0.0,
        }
    }

    pub async fn init_client(&self, client: &Client) {
        client
            .send_packet(&CInitializeWorldBorder::new(
                self.center_x,
                self.center_z,
                self.old_diameter,
                self.new_diameter,
                self.speed.into(),
                self.portal_teleport_boundary.into(),
                self.warning_blocks.into(),
                self.warning_time.into(),
            ))
            .await;
    }

    pub async fn set_center(&mut self, world: &World, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;

        world
            .broadcast_packet_all(&CSetBorderCenter::new(self.center_x, self.center_z))
            .await;
    }

    pub async fn set_diameter(&mut self, world: &World, diameter: f64, speed: Option<i64>) {
        self.old_diameter = self.new_diameter;
        self.new_diameter = diameter;

        match speed {
            Some(speed) => {
                world
                    .broadcast_packet_all(&CSetBorderLerpSize::new(
                        self.old_diameter,
                        self.new_diameter,
                        speed.into(),
                    ))
                    .await;
            }
            None => {
                world
                    .broadcast_packet_all(&CSetBorderSize::new(self.new_diameter))
                    .await;
            }
        }
    }

    pub async fn add_diameter(&mut self, world: &World, offset: f64, speed: Option<i64>) {
        self.set_diameter(world, self.new_diameter + offset, speed)
            .await;
    }

    pub async fn set_warning_delay(&mut self, world: &World, delay: i32) {
        self.warning_time = delay;

        world
            .broadcast_packet_all(&CSetBorderWarningDelay::new(self.warning_time.into()))
            .await;
    }

    pub async fn set_warning_distance(&mut self, world: &World, distance: i32) {
        self.warning_blocks = distance;

        world
            .broadcast_packet_all(&CSetBorderWarningDistance::new(self.warning_blocks.into()))
            .await;
    }
}
