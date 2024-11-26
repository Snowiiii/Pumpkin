use pumpkin_core::ProfileAction;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AuthenticationConfig {
    /// Whether to use Mojang authentication.
    pub enabled: bool,
    pub auth_url: Option<String>,
    pub prevent_proxy_connections: bool,
    pub prevent_proxy_connection_auth_url: Option<String>,
    /// Player profile handling.
    pub player_profile: PlayerProfileConfig,
    /// Texture handling.
    pub textures: TextureConfig,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prevent_proxy_connections: false,
            player_profile: Default::default(),
            textures: Default::default(),
            auth_url: None,
            prevent_proxy_connection_auth_url: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PlayerProfileConfig {
    /// Allow players flagged by Mojang (banned, forced name change).
    pub allow_banned_players: bool,
    /// Depends on the value above
    pub allowed_actions: Vec<ProfileAction>,
}

impl Default for PlayerProfileConfig {
    fn default() -> Self {
        Self {
            allow_banned_players: false,
            allowed_actions: vec![
                ProfileAction::ForcedNameChange,
                ProfileAction::UsingBannedSkin,
            ],
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct TextureConfig {
    /// Whether to use player textures.
    pub enabled: bool,

    pub allowed_url_schemes: Vec<String>,
    pub allowed_url_domains: Vec<String>,

    /// Specific texture types.
    pub types: TextureTypes,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_url_schemes: vec!["http".into(), "https".into()],
            allowed_url_domains: vec![".minecraft.net".into(), ".mojang.com".into()],
            types: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct TextureTypes {
    /// Use player skins.
    pub skin: bool,
    /// Use player capes.
    pub cape: bool,
    /// Use player elytras.
    /// (i didn't know myself that there are custom elytras)
    pub elytra: bool,
}

impl Default for TextureTypes {
    fn default() -> Self {
        Self {
            skin: true,
            cape: true,
            elytra: true,
        }
    }
}
