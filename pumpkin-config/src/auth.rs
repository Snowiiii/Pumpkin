use pumpkin_core::ProfileAction;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Deserialize, Serialize)]
pub struct AuthenticationConfig {
    /// Whether to use Mojang authentication.
    #[serde_inline_default(true)]
    pub enabled: bool,

    pub auth_url: String,

    /// Prevent proxy connections.
    #[serde_inline_default(false)]
    pub prevent_proxy_connections: bool,

    pub prevent_proxy_connection_auth_url: String,

    /// Player profile handling.
    #[serde(default)]
    pub player_profile: PlayerProfileConfig,

    /// Texture handling.
    #[serde(default)]
    pub textures: TextureConfig,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prevent_proxy_connections: false,
            player_profile: Default::default(),
            textures: Default::default(),
            auth_url: "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}".to_string(),
            prevent_proxy_connection_auth_url: "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}&ip={ip}".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PlayerProfileConfig {
    /// Allow players flagged by Mojang (banned, forced name change).
    pub allow_banned_players: bool,
    /// Depends on the value above
    #[serde(default = "default_allowed_actions")]
    pub allowed_actions: Vec<ProfileAction>,
}

fn default_allowed_actions() -> Vec<ProfileAction> {
    vec![
        ProfileAction::ForcedNameChange,
        ProfileAction::UsingBannedSkin,
    ]
}

impl Default for PlayerProfileConfig {
    fn default() -> Self {
        Self {
            allow_banned_players: false,
            allowed_actions: default_allowed_actions(),
        }
    }
}

#[serde_inline_default]
#[derive(Deserialize, Serialize)]
pub struct TextureConfig {
    /// Whether to use player textures.
    #[serde_inline_default(true)]
    pub enabled: bool,

    #[serde_inline_default(vec!["http".into(), "https".into()])]
    pub allowed_url_schemes: Vec<String>,
    #[serde_inline_default(vec![".minecraft.net".into(), ".mojang.com".into()])]
    pub allowed_url_domains: Vec<String>,

    /// Specific texture types.
    #[serde(default)]
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
#[serde_inline_default]
pub struct TextureTypes {
    /// Use player skins.
    #[serde_inline_default(true)]
    pub skin: bool,
    /// Use player capes.
    #[serde_inline_default(true)]
    pub cape: bool,
    /// Use player elytras.
    /// (i didn't know myself that there are custom elytras)
    #[serde_inline_default(true)]
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
