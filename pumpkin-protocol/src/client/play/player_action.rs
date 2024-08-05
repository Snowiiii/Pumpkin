use crate::Property;

pub enum PlayerAction {
    AddPlayer {
        name: String,
        properties: Vec<Property>,
    },
    InitializeChat(u8),
    UpdateGameMode(u8),
    UpdateListed {
        listed: bool,
    },
    UpdateLatency(u8),
    UpdateDisplayName(u8),
}
