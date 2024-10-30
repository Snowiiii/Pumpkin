use std::fmt::Display;

use fastnbt::SerOpts;
use num_derive::ToPrimitive;
use pumpkin_protocol::client::config::RegistryEntry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DamageTypeEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    death_message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effects: Option<String>,
    exhaustion: f32,
    message_id: String,
    scaling: String,
}

#[derive(Debug, ToPrimitive)]
pub enum DamageType {
    Arrow,
    BadRespawnPoint,
    Cactus,
    Campfire,
    Cramming,
    DragonBreath,
    Drown,
    DryOut,
    EnderPearl,
    Explosion,
    Fall,
    FallingAnvil,
    FallingBlock,
    FallingStalactite,
    Fireball,
    Fireworks,
    FlyIntoWall,
    Freeze,
    Generic,
    GenericKill,
    HotFloor,
    InFire,
    InWall,
    IndirectMagic,
    Lava,
    LightningBolt,
    MaceSmash,
    Magic,
    MobAttack,
    MobAttackNoAggro,
    MobProjectile,
    OnFire,
    OutOfWorld,
    OutsideBorder,
    PlayerAttack,
    PlayerExplosion,
    SonicBoom,
    Spit,
    Stalagmite,
    Starve,
    Sting,
    SweetBerryBush,
    Thorns,
    Thrown,
    Trident,
    UnattributedFireball,
    WindCharge,
    Wither,
    WitherSkull,
}

impl Display for DamageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DamageType::Arrow => "arrow",
            DamageType::BadRespawnPoint => "bad_respawn_point",
            DamageType::Cactus => "cactus",
            DamageType::Campfire => "campfire",
            DamageType::Cramming => "cramming",
            DamageType::DragonBreath => "dragon_breath",
            DamageType::Drown => "drown",
            DamageType::DryOut => "dry_out",
            DamageType::EnderPearl => "ender_pearl",
            DamageType::Explosion => "explosion",
            DamageType::Fall => "fall",
            DamageType::FallingAnvil => "falling_anvil",
            DamageType::FallingBlock => "falling_block",
            DamageType::FallingStalactite => "falling_stalactite",
            DamageType::Fireball => "fireball",
            DamageType::Fireworks => "fireworks",
            DamageType::FlyIntoWall => "fly_into_wall",
            DamageType::Freeze => "freeze",
            DamageType::Generic => "generic",
            DamageType::GenericKill => "generic_kill",
            DamageType::HotFloor => "hot_floor",
            DamageType::InFire => "in_fire",
            DamageType::InWall => "in_wall",
            DamageType::IndirectMagic => "indirect_magic",
            DamageType::Lava => "lava",
            DamageType::LightningBolt => "lightning_bolt",
            DamageType::MaceSmash => "mace_smash",
            DamageType::Magic => "magic",
            DamageType::MobAttack => "mob_attack",
            DamageType::MobAttackNoAggro => "mob_attack_no_aggro",
            DamageType::MobProjectile => "mob_projectile",
            DamageType::OnFire => "on_fire",
            DamageType::OutOfWorld => "out_of_world",
            DamageType::OutsideBorder => "outside_border",
            DamageType::PlayerAttack => "player_attack",
            DamageType::PlayerExplosion => "player_explosion",
            DamageType::SonicBoom => "sonic_boom",
            DamageType::Spit => "spit",
            DamageType::Stalagmite => "stalagmite",
            DamageType::Starve => "starve",
            DamageType::Sting => "sting",
            DamageType::SweetBerryBush => "sweet_berry_bush",
            DamageType::Thorns => "thorns",
            DamageType::Thrown => "thrown",
            DamageType::Trident => "trident",
            DamageType::UnattributedFireball => "unattributed_fireball",
            DamageType::WindCharge => "wind_charge",
            DamageType::Wither => "wither",
            DamageType::WitherSkull => "wither_skull",
        };
        write!(f, "{}", name)
    }
}

const NAMES: &[&str] = &[
    "arrow",
    "bad_respawn_point",
    "cactus",
    "campfire",
    "cramming",
    "dragon_breath",
    "drown",
    "dry_out",
    "ender_pearl",
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
    "mace_smash",
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
    "wind_charge",
    "wither",
    "wither_skull",
];

pub(super) fn entries() -> Vec<RegistryEntry<'static>> {
    let items: Vec<_> = NAMES
        .iter()
        .map(|name| RegistryEntry {
            entry_id: name,
            data: fastnbt::to_bytes_with_opts(
                &DamageTypeEntry {
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
