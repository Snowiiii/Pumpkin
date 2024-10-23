use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::DisplayScoreboard as i32)]
pub struct CDisplayObjective<'a> {
    position: VarInt,
    score_name: &'a str,
}

impl<'a> CDisplayObjective<'a> {
    pub fn new(position: DisplaySlot, score_name: &'a str) -> Self {
        Self {
            position: VarInt(position as i32),
            score_name,
        }
    }
}

#[repr(i32)]
pub enum DisplaySlot {
    List,
    Sidebar,
    BelowName,
    TeamBlack,
    TeamDarkBlue,
    TeamDarkGreen,
    TeanDarkAqua,
    TeamDarkRed,
    TeamDarkPurple,
    TeamGold,
    TeamGray,
    TeamDarkGray,
    TeamBlue,
    TeamGreen,
    TeamAqua,
    TeamRed,
    TeamLightPurple,
    TeamYellow,
    TeamWhite,
}
