use fastnbt::SerOpts;
use pumpkin_protocol::client::config::RegistryEntry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DamageType {
    #[serde(skip_serializing_if = "Option::is_none")]
    death_message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effects: Option<String>,
    exhaustion: f32,
    message_id: String,
    scaling: String,
}

const NAMES: &[&str] = &[
    "arrow",
    "bad_respawn_point",
    "cactus",
    "cramming",
    "campfire",
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

pub(super) fn entries() -> Vec<RegistryEntry<'static>> {
    let items: Vec<_> = NAMES
        .iter()
        .map(|name| RegistryEntry {
            entry_id: name,
            data: fastnbt::to_bytes_with_opts(
                &DamageType {
                    exhaustion: 0.1,
                    message_id: "inFire".into(),
                    scaling: "when_caused_by_living_non_player".into(),
                    death_message_type: None,
                    effects: None,
                },
                SerOpts::network_nbt(),
            )
            .unwrap(),
        })
        .collect();

    items
}
