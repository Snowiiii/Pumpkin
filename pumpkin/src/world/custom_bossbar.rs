use crate::command::args::GetCloned;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions};
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
    #[deny(clippy::new_without_default)]
    #[must_use]
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

impl Default for CustomBossbars {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomBossbars {
    #[must_use]
    pub fn new() -> CustomBossbars {
        Self {
            custom_bossbars: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get_player_bars(&self, uuid: &Uuid) -> Option<Vec<&Bossbar>> {
        let mut player_bars: Vec<&Bossbar> = Vec::new();
        for bossbar in &self.custom_bossbars {
            if bossbar.1.player.contains(uuid) {
                player_bars.push(&bossbar.1.bossbar_data);
            }
        }
        if !player_bars.is_empty() {
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

    pub fn replace_bossbar(&mut self, resource_location: &str, bossbar_data: CustomBossbar) {
        self.custom_bossbars
            .insert(resource_location.to_string(), bossbar_data);
    }

    #[must_use]
    pub fn get_all_bossbars(&self) -> Option<Vec<CustomBossbar>> {
        let mut bossbars: Vec<CustomBossbar> = Vec::new();
        for bossbar in self.custom_bossbars.clone() {
            bossbars.push(bossbar.1);
        }
        Some(bossbars)
    }

    #[must_use]
    pub fn get_bossbar(&self, resource_location: &str) -> Option<CustomBossbar> {
        let bossbar = self.custom_bossbars.get(resource_location);
        if let Some(bossbar) = bossbar {
            return Some(bossbar.clone());
        }
        None
    }

    pub async fn remove_bossbar(
        &mut self,
        server: &Server,
        resource_location: String,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_cloned(&resource_location);
        if let Some(bossbar) = bossbar {
            self.custom_bossbars.remove(&resource_location);

            let players: Vec<Arc<Player>> = server.get_all_players().await;

            let online_players: Vec<&Arc<Player>> = players
                .iter()
                .filter(|player| bossbar.player.contains(&player.gameprofile.id))
                .collect();

            if bossbar.visible {
                for player in online_players {
                    player.remove_bossbar(bossbar.bossbar_data.uuid).await;
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    #[must_use]
    pub fn has_bossbar(&self, resource_location: &str) -> bool {
        self.custom_bossbars.contains_key(resource_location)
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

            let ratio = f64::from(value) / f64::from(max_value);
            let health: f32;

            if ratio >= 1.0 {
                health = 1.0;
            } else if ratio <= 0.0 {
                health = 0.0;
            } else {
                health = ratio as f32;
            }

            bossbar.value = value;
            bossbar.max = max_value;
            bossbar.bossbar_data.health = health;

            if !bossbar.visible {
                return Ok(());
            }

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
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub async fn update_visibility(
        &mut self,
        server: &Server,
        resource_location: String,
        new_visibility: bool,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.visible == new_visibility && new_visibility {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "The bossbar is already visible",
                )));
            }

            if bossbar.visible == new_visibility && !new_visibility {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "The bossbar is already hidden",
                )));
            }

            bossbar.visible = new_visibility;

            let players: Vec<Arc<Player>> = server.get_all_players().await;
            let online_players: Vec<&Arc<Player>> = players
                .iter()
                .filter(|player| bossbar.player.contains(&player.gameprofile.id))
                .collect();

            for player in online_players {
                if bossbar.visible {
                    player.send_bossbar(&bossbar.bossbar_data).await;
                } else {
                    player.remove_bossbar(bossbar.bossbar_data.uuid).await;
                }
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub async fn update_name(
        &mut self,
        server: &Server,
        resource_location: &str,
        new_title: &str,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.bossbar_data.title == new_title {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "That's already the name of this bossbar",
                )));
            }

            bossbar.bossbar_data.title = new_title.to_string();

            if !bossbar.visible {
                return Ok(());
            }

            let players: Vec<Arc<Player>> = server.get_all_players().await;
            let matching_players: Vec<&Arc<Player>> = players
                .iter()
                .filter(|player| bossbar.player.contains(&player.gameprofile.id))
                .collect();
            for player in matching_players {
                player
                    .update_bossbar_title(
                        bossbar.bossbar_data.uuid,
                        bossbar.bossbar_data.title.clone(),
                    )
                    .await;
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location.to_string(),
        ))
    }

    pub async fn update_color(
        &mut self,
        server: &Server,
        resource_location: String,
        new_color: BossbarColor,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.bossbar_data.color == new_color {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "That's already the color of this bossbar",
                )));
            }

            bossbar.bossbar_data.color = new_color;

            if !bossbar.visible {
                return Ok(());
            }

            let players: Vec<Arc<Player>> = server.get_all_players().await;
            let matching_players: Vec<&Arc<Player>> = players
                .iter()
                .filter(|player| bossbar.player.contains(&player.gameprofile.id))
                .collect();
            for player in matching_players {
                player
                    .update_bossbar_style(
                        bossbar.bossbar_data.uuid,
                        bossbar.bossbar_data.color.clone(),
                        bossbar.bossbar_data.division.clone(),
                    )
                    .await;
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub async fn update_division(
        &mut self,
        server: &Server,
        resource_location: String,
        new_division: BossbarDivisions,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            if bossbar.bossbar_data.division == new_division {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "That's already the style of this bossbar",
                )));
            }

            bossbar.bossbar_data.division = new_division;

            if !bossbar.visible {
                return Ok(());
            }

            let players: Vec<Arc<Player>> = server.get_all_players().await;
            let matching_players: Vec<&Arc<Player>> = players
                .iter()
                .filter(|player| bossbar.player.contains(&player.gameprofile.id))
                .collect();
            for player in matching_players {
                player
                    .update_bossbar_style(
                        bossbar.bossbar_data.uuid,
                        bossbar.bossbar_data.color.clone(),
                        bossbar.bossbar_data.division.clone(),
                    )
                    .await;
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }

    pub async fn update_players(
        &mut self,
        server: &Server,
        resource_location: String,
        new_players: Vec<Uuid>,
    ) -> Result<(), BossbarUpdateError> {
        let bossbar = self.custom_bossbars.get_mut(&resource_location);
        if let Some(bossbar) = bossbar {
            // Get difference between old and new player list and remove bossbars from old players
            let removed_players: Vec<Uuid> = bossbar
                .player
                .iter()
                .filter(|item| !new_players.contains(item))
                .copied()
                .collect();

            let added_players: Vec<Uuid> = new_players
                .iter()
                .filter(|item| !bossbar.player.contains(item))
                .copied()
                .collect();

            if removed_players.is_empty() && added_players.is_empty() {
                return Err(BossbarUpdateError::NoChanges(String::from(
                    "Those players are already on the bossbar with nobody to add or remove",
                )));
            }

            if bossbar.visible {
                for uuid in removed_players {
                    let Some(player) = server.get_player_by_uuid(uuid).await else {
                        continue;
                    };

                    player.remove_bossbar(bossbar.bossbar_data.uuid).await;
                }
            }

            bossbar.player = new_players;

            if !bossbar.visible {
                return Ok(());
            }

            for uuid in added_players {
                let Some(player) = server.get_player_by_uuid(uuid).await else {
                    continue;
                };

                player.send_bossbar(&bossbar.bossbar_data).await;
            }

            return Ok(());
        }
        Err(BossbarUpdateError::InvalidResourceLocation(
            resource_location,
        ))
    }
}
