use std::collections::HashMap;

use pumpkin_core::text::TextComponent;
use pumpkin_protocol::{
    client::play::{CDisplayObjective, CUpdateObjectives, CUpdateScore, RenderType},
    NumberFormat, VarInt,
};

use super::World;

#[derive(Default, Debug)]
pub struct Scoreboard {
    objectives: HashMap<String, ScoreboardObjective<'static>>,
    //  teams: HashMap<String, Team>,
}

impl Scoreboard {
    #[must_use]
    pub fn new() -> Self {
        Self {
            objectives: HashMap::new(),
        }
    }

    pub async fn add_objective<'a>(&mut self, world: &World, objective: ScoreboardObjective<'a>) {
        if self.objectives.contains_key(objective.name) {
            // Maybe make this an error ?
            log::warn!(
                "Tried to create Objective which does already exist, {}",
                &objective.name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateObjectives::new(
                objective.name,
                pumpkin_protocol::client::play::Mode::Add,
                objective.display_name,
                objective.render_type,
                objective.number_format,
            ))
            .await;
        world
            .broadcast_packet_all(&CDisplayObjective::new(
                pumpkin_protocol::client::play::DisplaySlot::Sidebar,
                objective.name,
            ))
            .await;
    }

    pub async fn update_score<'a>(&self, world: &World, score: ScoreboardScore<'a>) {
        if self.objectives.contains_key(score.objective_name) {
            log::warn!(
                "Tried to place a score into a Objective which does not exist, {}",
                &score.objective_name
            );
            return;
        }
        world
            .broadcast_packet_all(&CUpdateScore::new(
                score.entity_name,
                score.objective_name,
                score.value,
                score.display_name,
                score.number_format,
            ))
            .await;
    }

    // pub fn add_team(&mut self, name: String) {
    //     if self.teams.contains_key(&name) {
    //         // Maybe make this an error ?
    //         log::warn!("Tried to create Team which does already exist, {}", name);
    //     }
    // }
}

#[derive(Debug)]
pub struct ScoreboardObjective<'a> {
    name: &'a str,
    display_name: TextComponent<'a>,
    render_type: RenderType,
    number_format: Option<NumberFormat<'a>>,
}

impl<'a> ScoreboardObjective<'a> {
    #[must_use]
    pub const fn new(
        name: &'a str,
        display_name: TextComponent<'a>,
        render_type: RenderType,
        number_format: Option<NumberFormat<'a>>,
    ) -> Self {
        Self {
            name,
            display_name,
            render_type,
            number_format,
        }
    }
}

pub struct ScoreboardScore<'a> {
    entity_name: &'a str,
    objective_name: &'a str,
    value: VarInt,
    display_name: Option<TextComponent<'a>>,
    number_format: Option<NumberFormat<'a>>,
}

impl<'a> ScoreboardScore<'a> {
    #[must_use]
    pub const fn new(
        entity_name: &'a str,
        objective_name: &'a str,
        value: VarInt,
        display_name: Option<TextComponent<'a>>,
        number_format: Option<NumberFormat<'a>>,
    ) -> Self {
        Self {
            entity_name,
            objective_name,
            value,
            display_name,
            number_format,
        }
    }
}
