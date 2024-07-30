use super::CodecItem;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DamageType {
    exhaustion: f32,
    message_id: String,
    scaling: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    effects: Option<String>,
}

const NAMES: &[&str] = &[
    "in_fire",
    "lightning_bolt",
    "on_fire",
    "lava",
    "hot_floor",
    "in_wall",
    "cramming",
    "drown",
    "starve",
    "cactus",
    "fall",
    "fly_into_wall",
    "out_of_world",
    "generic",
    "magic",
    "wither",
    "dragon_breath",
    "dry_out",
    "sweet_berry_bush",
    "freeze",
    "stalagmite",
    // 1.20+
    "outside_border",
    "generic_kill",
];

pub(super) fn all() -> Vec<CodecItem<DamageType>> {
    let mut items: Vec<_> = NAMES
        .iter()
        .map(|name| CodecItem {
            name: (*name).into(),
            id: 0,
            element: DamageType {
                exhaustion: 0.1,
                message_id: "inFire".into(),
                scaling: "when_caused_by_living_non_player".into(),
                effects: None,
            },
        })
        .collect();

    items[1].element.effects = Some("burning".into());

    items
}
