use crate::entity::player::Player;
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::client::play::{BosseventAction, CBossEvent};
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub enum BossbarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Clone, PartialEq)]
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
    pub flags: BossbarFlags,
}

impl Bossbar {
    pub fn new(title: String) -> Bossbar {
        let uuid = Uuid::new_v4();

        Self {
            uuid,
            title,
            health: 0.0,
            color: BossbarColor::White,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::NoFlags,
        }
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
        let boss_action = BosseventAction::Remove;

        let packet = CBossEvent::new(uuid, boss_action);
        self.client.send_packet(&packet).await;
    }

    pub async fn update_bossbar_health(&self, uuid: Uuid, health: f32) {
        let boss_action = BosseventAction::UpdateHealth(health);

        let packet = CBossEvent::new(uuid, boss_action);
        self.client.send_packet(&packet).await;
    }

    pub async fn update_bossbar_title(&self, uuid: Uuid, title: String) {
        let boss_action = BosseventAction::UpdateTile(TextComponent::text_string(title));

        let packet = CBossEvent::new(uuid, boss_action);
        self.client.send_packet(&packet).await;
    }

    pub async fn update_bossbar_style(
        &self,
        uuid: Uuid,
        color: BossbarColor,
        dividers: BossbarDivisions,
    ) {
        let boss_action = BosseventAction::UpdateStyle {
            color: (color as u8).into(),
            dividers: (dividers as u8).into(),
        };

        let packet = CBossEvent::new(uuid, boss_action);
        self.client.send_packet(&packet).await;
    }

    pub async fn update_bossbar_flags(&self, uuid: Uuid, flags: BossbarFlags) {
        let boss_action = BosseventAction::UpdateFlags(flags as u8);

        let packet = CBossEvent::new(uuid, boss_action);
        self.client.send_packet(&packet).await;
    }
}
