use std::collections::HashMap;
use serde::Serialize;
use uuid::Uuid;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::{BosseventAction, CBossEvent};
use crate::entity::player::Player;

#[derive(Clone)]
pub enum BossbarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Clone)]
pub enum BossbarDivisions {
    NoDivision,
    Notches6,
    Notches10,
    Notches12,
    Notches20,
}

#[derive(Clone)]
pub enum BossbarFlags {
    NoFlags,
    DarkenSky = 0x01,
    DragonBar = 0x02,
    CreateFog = 0x04,
}

#[derive(Clone)]
pub struct Bossbar {
    pub uuid: Uuid,
    pub title: String,
    pub health: f32,
    pub color: BossbarColor,
    pub division: BossbarDivisions,
    pub flags: BossbarFlags
}

impl Bossbar {
    pub fn new(
        title: String,

    ) -> Bossbar {
        let uuid = Uuid::new_v4();

        Self {
            uuid: uuid,
            title: title,
            health: 0.0,
            color: BossbarColor::White,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::NoFlags
        }
    }
}

//TODO: Remove after debugging
impl Default for Bossbar {
    fn default() -> Self {
        Self::new(String::from("1"))
    }
}

/// Representing the stored custom boss bars from level.dat
pub struct CustomBossbar {
    pub namespace: String,
    pub bossbar_data: Bossbar,
    pub player: Vec<Uuid>,
}

pub struct CustomBossbars {
    custom_bossbars: HashMap<String, CustomBossbar>,
}

impl CustomBossbars {
    pub fn new() -> CustomBossbars {

        let mut example_data: HashMap<String, CustomBossbar> = HashMap::new();
        let mut players: Vec<Uuid> = Vec::new();
        players.push(Uuid::from_u128(0x1DCC7E94EA424A0394D0752889039383));
        example_data.insert("minecraft:123".to_string(), CustomBossbar {namespace: "minecraft:123".to_string(), bossbar_data: Bossbar::default(), player: Vec::new()});

        Self {
            custom_bossbars: example_data,
        }
    }

    pub fn get_player_bars(&self, uuid: &Uuid) -> Option<Vec<&Bossbar>> {
        let mut player_bars: Vec<&Bossbar> = Vec::new();
        for bossbar in &self.custom_bossbars {
            // if(bossbar.player.contains(&uuid)) {
            //     player_bars.push(&bossbar.bossbar_data);
            // }
            player_bars.push(&bossbar.1.bossbar_data);
        }
        if(player_bars.len() > 0) {
            return Some(player_bars);
        }
        None
    }

    pub fn create_bossbar(&mut self, namespace: String, bossbar_data: Bossbar) {
        self.custom_bossbars.insert(namespace.clone(), CustomBossbar {namespace, bossbar_data, player: Vec::new()});
    }
}

/// Extension of the player to send the manage the bossbar
impl Player {
    pub async fn send_bossbar(&self, bossbar: &Bossbar) {
        // Maybe this section could be implemented. feel free to change
        let bossbar = bossbar.clone();
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