use super::CodecItem;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DamageType {
    message_id: String,
    scaling: String,
    exhaustion: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    effects: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    death_message_type: Option<String>,
}

const NAMES: &[&str] = &[
    "arrow",
    "bad_respawn_point",
    "cactus",
    "cramming",
    "dragon_breath",
    "drown",
    "dry_out",
    "explosion",
    "fall",
    "falling_anvil",
    "falling_block",
    "falling_stalactite",
    "fireball",
    "fireworks",
    "fly_into_wall",
    "freeze",
    "generic",
    "generic_kill",
    "hot_floor",
    "in_fire",
    "in_wall",
    "indirect_magic",
    "lava",
    "lightning_bolt",
    "magic",
    "mob_attack",
    "mob_attack_no_aggro",
    "mob_projectile",
    "on_fire",
    "out_of_world",
    "outside_border",
    "player_attack",
    "player_explosion",
    "sonic_boom",
    "spit",
    "stalagmite",
    "starve",
    "sting",
    "sweet_berry_bush",
    "thorns",
    "thrown",
    "trident",
    "unattributed_fireball",
    "wither",
    "wither_skull",
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
                death_message_type: Some("default".into()),
            },
        })
        .collect();

    items[1].element.effects = Some("burning".into());

    items
}
