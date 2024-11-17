use serde::Serialize;
use uuid::Uuid;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::{BosseventAction, CBossEvent};
use crate::entity::player::Player;

pub enum BossbarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

pub enum BossbarDivisions {
    NoDivision,
    Notches6,
    Notches10,
    Notches12,
    Notches20,
}

pub enum BossbarFlags {
    DarkenSky = 0x01,
    DragonBar = 0x02,
    CreateFog = 0x04,
}

pub struct Bossbar {
    pub uuid: Uuid,
    pub title: String,
    pub health: f32,
    pub color: BossbarColor,
    pub division: BossbarDivisions,
    pub flags: BossbarFlags
}

impl Player {
    pub async fn send_bossbar(&self, bossbar: Bossbar) {
        // Maybe this section could be implemented. feel free to change
        let boss_action = BosseventAction::Add {
            title: TextComponent::text_string(bossbar.title),
            health: bossbar.health,
            color: (bossbar.color as u8).into(),
            division: (bossbar.division as u8).into(),
            flags: bossbar.flags as u8,
        };
        
        let packet = CBossEvent::new(bossbar.uuid, boss_action);
        self.client.send_packet(&packet).await;
    }
    pub async fn remove_bossbar(&self, uuid: Uuid) {
        todo!()
    }
    
    pub async fn update_bossbar_health(&self, uuid: Uuid, health: f32) {
        todo!()
    }
    
    pub async fn update_bossbar_title(&self, uuid: Uuid, title: String) {
        todo!()
    }
    
    pub async fn update_bossbar_style(&self, uuid: Uuid, color: BossbarColor, dividers: BossbarDivisions) {
        todo!()
    }
    
    pub async fn update_bossbar_flags(&self, uuid: Uuid, flags: BossbarFlags) {
        todo!()
    }
}