use pumpkin_protocol::codec::identifier::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimPattern {
    asset_id: Identifier,
    template_item: String,
    //  description: TextComponent<'static>,
    decal: bool,
}
