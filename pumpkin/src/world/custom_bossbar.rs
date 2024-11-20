use crate::entity::player::Player;
use crate::server::Server;
use crate::world::bossbar::Bossbar;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BossbarUpdateError {
    #[error("Invalid resource location")]
    InvalidResourceLocation(String),
    #[error("No changes")]
    NoChanges(String),
}

/// Representing the stored custom boss bars from level.dat
#[derive(Clone)]
pub struct CustomBossbar {
    pub namespace: String,
    pub bossbar_data: Bossbar,
    pub max: u32,
    pub value: u32,
    pub visible: bool,
    pub player: Vec<Uuid>,
}

impl CustomBossbar {
    pub fn new(namespace: String, bossbar_data: Bossbar) -> Self {
        Self {
            namespace,
            bossbar_data,
            max: 100,
            value: 0,
            visible: true,
            player: vec![],
        }
    }
}

pub struct CustomBossbars {
    pub custom_bossbars: HashMap<String, CustomBossbar>,
}

impl CustomBossbars {
    pub fn new() -> CustomBossbars {
        //TODO: Remove after debugging
        let mut example_data: HashMap<String, CustomBossbar> = HashMap::new();
        let mut players: Vec<Uuid> = Vec::new();
        players.push(Uuid::from_u128(0x1DCC7E94EA424A0394D0752889039383));
        example_data.insert(
            "minecraft:123".to_string(),
            CustomBossbar::new("minecraft:123".to_string(), Bossbar::default()),
        );

        Self {
            custom_bossbars: example_data,
        }
    }

    pub fn get_player_bars(&self, uuid: &Uuid) -> Option<Vec<&Bossbar>> {
        let mut player_bars: Vec<&Bossbar> = Vec::new();
        for bossbar in &self.custom_bossbars {
            if bossbar.1.player.contains(&uuid) {
                player_bars.push(&bossbar.1.bossbar_data);
            }
        }
        if player_bars.len() > 0 {
            return Some(player_bars);
        }
        None
    }

    pub fn create_bossbar(&mut self, namespace: String, bossbar_data: Bossbar) {
        self.custom_bossbars.insert(
            namespace.clone(),
            CustomBossbar::new(namespace, bossbar_data),
        );
    }

    pub fn replace_bossbar(&mut self, namespace: String, bossbar_data: CustomBossbar) {
        self.custom_bossbars.insert(namespace.clone(), bossbar_data);
    }

    pub fn get_all_bossbars(&self) -> Option<Vec<CustomBossbar>> {
        let mut bossbars: Vec<CustomBossbar> = Vec::new();
        for bossbar in self.custom_bossbars.clone() {
            bossbars.push(bossbar.1);
        }
        Some(bossbars)
    }

    pub fn get_bossbar(&self, resource_location: String) -> Option<CustomBossbar> {
        let bossbar = self.custom_bossbars.get(&resource_location);
        if let Some(bossbar) = bossbar {
            return Some(bossbar.clone());
        }
        None
    }

    pub fn remove_bossbar(&mut self, namespace: String) {
        self.custom_bossbars.remove(&namespace);
    }

    pub fn has_bossbar(&self, namespace: String) -> bool {
        self.custom_bossbars.contains_key(&namespace)
    }

    pub async fn update_health(
        &mut self,
        server: &Server,
        resource_location: String,
        max_value: u32,
        value: u32,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.value == value && bossbar.max == max_value {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "That's already the value of this bossbar",
                )));
            }

            bossbar.value = value;
            bossbar.max = max_value;
            bossbar.bossbar_data.health = value as f32 / max_value as f32;

            let players: Vec<Arc<Player>> = server.get_all_players().await;
            let matching_players: Vec<&Arc<Player>> = players
                .iter()
                .filter(|player| bossbar.player.contains(&player.gameprofile.id))
                .collect();
            for player in matching_players {
                player
                    .update_bossbar_health(bossbar.bossbar_data.uuid, bossbar.bossbar_data.health)
                    .await;
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(resource_location))
    }

    pub async fn update_players(
        &mut self,
        server: &Server,
        resource_location: String,
        new_players: Vec<Uuid>,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            // Get differnce between old and new player list and remove bossbars from old players
            let removed_players: Vec<Uuid> = bossbar
                .player
                .iter()
                .filter(|item| !new_players.contains(item))
                .cloned()
                .collect();

            let added_players: Vec<Uuid> = new_players
                .iter()
                .filter(|item| !bossbar.player.contains(item))
                .cloned()
                .collect();

            if removed_players.len() == 0 && added_players.len() == 0 {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "Those players are already on the bossbar with nobody to add or remove",
                )));
            }

            for uuid in removed_players {
                let Some(player) = server.get_player_by_uuid(uuid).await else {
                    continue;
                };

                player.remove_bossbar(bossbar.bossbar_data.uuid).await;
            }

            bossbar.player = new_players;

            for uuid in added_players {
                let Some(player) = server.get_player_by_uuid(uuid).await else {
                    continue;
                };

                player.send_bossbar(&bossbar.bossbar_data).await;
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(resource_location))
    }
}
