use std::sync::LazyLock;

use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_core::text::TextComponent;
use pumpkin_protocol::{
    client::{
        config::{CConfigAddResourcePack, CConfigServerLinks, CKnownPacks},
        login::{CLoginSuccess, CSetCompression},
    },
    codec::var_int::VarInt,
    server::login::{SEncryptionResponse, SLoginCookieResponse, SLoginPluginResponse, SLoginStart},
    ConnectionState, KnownPack, Label, Link, LinkType,
};
use uuid::Uuid;

use crate::{
    net::{
        authentication::{self, AuthError},
        offline_uuid,
        packet::is_valid_player_name,
        proxy::{bungeecord, velocity},
        Client, GameProfile,
    },
    server::Server,
};

static LINKS: LazyLock<Vec<Link>> = LazyLock::new(|| {
    let mut links: Vec<Link> = Vec::new();

    let bug_report = &ADVANCED_CONFIG.server_links.bug_report;
    if !bug_report.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::BugReport), bug_report));
    }

    let support = &ADVANCED_CONFIG.server_links.support;
    if !support.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::Support), support));
    }

    let status = &ADVANCED_CONFIG.server_links.status;
    if !status.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::Status), status));
    }

    let feedback = &ADVANCED_CONFIG.server_links.feedback;
    if !feedback.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::Feedback), feedback));
    }

    let community = &ADVANCED_CONFIG.server_links.community;
    if !community.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::Community), community));
    }

    let website = &ADVANCED_CONFIG.server_links.website;
    if !website.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::Website), website));
    }

    let forums = &ADVANCED_CONFIG.server_links.forums;
    if !forums.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::Forums), forums));
    }

    let news = &ADVANCED_CONFIG.server_links.news;
    if !news.is_empty() {
        links.push(Link::new(Label::BuiltIn(LinkType::News), news));
    }

    let announcements = &ADVANCED_CONFIG.server_links.announcements;
    if !announcements.is_empty() {
        links.push(Link::new(
            Label::BuiltIn(LinkType::Announcements),
            announcements,
        ));
    }

    for (key, value) in &ADVANCED_CONFIG.server_links.custom {
        links.push(Link::new(
            Label::TextComponent(TextComponent::text(key)),
            value,
        ));
    }
    links
});

impl Client {
    pub async fn handle_login_start(&self, server: &Server, login_start: SLoginStart) {
        log::debug!("login start");

        // Don't allow new logons when server is full.
        // If max players is set to zero, then there is no max player count enforced.
        // TODO: If client is an operator or otherwise suitable elevated permissions, allow client to bypass this requirement.
        let max_players = BASIC_CONFIG.max_players;
        if max_players > 0 && server.get_player_count().await >= max_players as usize {
            self.kick("The server is currently full, please try again later")
                .await;
            return;
        }

        if !is_valid_player_name(&login_start.name) {
            self.kick("Invalid characters in username").await;
            return;
        }
        // default game profile, when no online mode
        // TODO: make offline uuid
        let mut gameprofile = self.gameprofile.lock().await;
        let proxy = &ADVANCED_CONFIG.proxy;
        if proxy.enabled {
            if proxy.velocity.enabled {
                velocity::velocity_login(self).await;
            } else if proxy.bungeecord.enabled {
                match bungeecord::bungeecord_login(
                    &self.address,
                    &self.server_address.lock().await,
                    login_start.name,
                )
                .await
                {
                    Ok((_ip, profile)) => {
                        // self.address.lock() = ip;
                        self.finish_login(&profile).await;
                        *gameprofile = Some(profile);
                    }
                    Err(error) => self.kick(&error.to_string()).await,
                }
            }
        } else {
            let id = if BASIC_CONFIG.online_mode {
                login_start.uuid
            } else {
                offline_uuid(&login_start.name).expect("This is very not safe and bad")
            };

            let profile = GameProfile {
                id,
                name: login_start.name,
                properties: vec![],
                profile_actions: None,
            };

            if BASIC_CONFIG.encryption {
                let verify_token: [u8; 4] = rand::random();
                self.send_packet(
                    &server.encryption_request(&verify_token, BASIC_CONFIG.online_mode),
                )
                .await;
            } else {
                if ADVANCED_CONFIG.packet_compression.enabled {
                    self.enable_compression().await;
                }
                self.finish_login(&profile).await;
            }

            *gameprofile = Some(profile);
        }
    }

    pub async fn handle_encryption_response(
        &self,
        server: &Server,
        encryption_response: SEncryptionResponse,
    ) {
        log::debug!("Handling encryption");
        let shared_secret = server.decrypt(&encryption_response.shared_secret).unwrap();

        if let Err(error) = self.set_encryption(Some(&shared_secret)).await {
            self.kick(&error.to_string()).await;
            return;
        }

        let mut gameprofile = self.gameprofile.lock().await;

        let Some(profile) = gameprofile.as_mut() else {
            self.kick("No Game profile").await;
            return;
        };

        if BASIC_CONFIG.online_mode {
            // Online mode auth
            match self
                .authenticate(server, &shared_secret, &profile.name)
                .await
            {
                Ok(new_profile) => *profile = new_profile,
                Err(e) => {
                    self.kick(&e.to_string()).await;
                    return;
                }
            }
        }

        // Don't allow duplicate UUIDs
        if let Some(online_player) = &server.get_player_by_uuid(profile.id).await {
            log::debug!("Player (IP '{}', username '{}') tried to log in with the same UUID ('{}') as an online player (IP '{}', username '{}')", &self.address.lock().await, &profile.name, &profile.id, &online_player.client.address.lock().await, &online_player.gameprofile.name);
            self.kick("You are already connected to this server").await;
            return;
        }

        // Don't allow a duplicate username
        if let Some(online_player) = &server.get_player_by_name(&profile.name).await {
            log::debug!("A player (IP '{}', attempted username '{}') tried to log in with the same username as an online player (UUID '{}', IP '{}', username '{}')", &self.address.lock().await, &profile.name, &profile.id, &online_player.client.address.lock().await, &online_player.gameprofile.name);
            self.kick("A player with this username is already connected")
                .await;
            return;
        }

        if ADVANCED_CONFIG.packet_compression.enabled {
            self.enable_compression().await;
        }
        self.finish_login(profile).await;
    }

    async fn enable_compression(&self) {
        let compression = ADVANCED_CONFIG.packet_compression.compression_info.clone();
        self.send_packet(&CSetCompression::new(compression.threshold.into()))
            .await;
        self.set_compression(Some(compression)).await;
    }

    async fn finish_login(&self, profile: &GameProfile) {
        let packet = CLoginSuccess::new(&profile.id, &profile.name, &profile.properties);
        self.send_packet(&packet).await;
    }

    async fn authenticate(
        &self,
        server: &Server,
        shared_secret: &[u8],
        username: &str,
    ) -> Result<GameProfile, AuthError> {
        if let Some(auth_client) = &server.auth_client {
            let hash = server.digest_secret(shared_secret);
            let ip = self.address.lock().await.ip();
            let profile = authentication::authenticate(username, &hash, &ip, auth_client).await?;

            // Check if player should join
            if let Some(actions) = &profile.profile_actions {
                if ADVANCED_CONFIG
                    .authentication
                    .player_profile
                    .allow_banned_players
                {
                    for allowed in &ADVANCED_CONFIG
                        .authentication
                        .player_profile
                        .allowed_actions
                    {
                        if !actions.contains(allowed) {
                            return Err(AuthError::DisallowedAction);
                        }
                    }
                    if !actions.is_empty() {
                        return Err(AuthError::Banned);
                    }
                } else if !actions.is_empty() {
                    return Err(AuthError::Banned);
                }
            }
            // validate textures
            for property in &profile.properties {
                authentication::validate_textures(
                    property,
                    &ADVANCED_CONFIG.authentication.textures,
                )
                .map_err(AuthError::TextureError)?;
            }
            return Ok(profile);
        }
        Err(AuthError::MissingAuthClient)
    }

    pub fn handle_login_cookie_response(&self, packet: SLoginCookieResponse) {
        // TODO: allow plugins to access this
        log::debug!(
        "Received cookie_response[login]: key: \"{}\", has_payload: \"{}\", payload_length: \"{}\"",
        packet.key.to_string(),
        packet.has_payload,
        packet.payload_length.unwrap_or(VarInt::from(0)).0
    );
    }
    pub async fn handle_plugin_response(&self, plugin_response: SLoginPluginResponse) {
        log::debug!("Handling plugin");
        let velocity_config = &ADVANCED_CONFIG.proxy.velocity;
        if velocity_config.enabled {
            let mut address = self.address.lock().await;
            match velocity::receive_velocity_plugin_response(
                address.port(),
                velocity_config,
                plugin_response,
            ) {
                Ok((profile, new_address)) => {
                    self.finish_login(&profile).await;
                    *self.gameprofile.lock().await = Some(profile);
                    *address = new_address;
                }
                Err(error) => self.kick(&error.to_string()).await,
            }
        }
    }

    pub async fn handle_login_acknowledged(&self, server: &Server) {
        log::debug!("Handling login acknowledged");
        self.connection_state.store(ConnectionState::Config);
        self.send_packet(&server.get_branding()).await;

        let resource_config = &ADVANCED_CONFIG.resource_pack;
        if resource_config.enabled {
            let resource_pack = CConfigAddResourcePack::new(
                Uuid::new_v3(
                    &uuid::Uuid::NAMESPACE_DNS,
                    resource_config.resource_pack_url.as_bytes(),
                ),
                &resource_config.resource_pack_url,
                &resource_config.resource_pack_sha1,
                resource_config.force,
                if resource_config.prompt_message.is_empty() {
                    None
                } else {
                    Some(TextComponent::text(&resource_config.prompt_message))
                },
            );

            self.send_packet(&resource_pack).await;
        }

        if ADVANCED_CONFIG.server_links.enabled {
            self.send_packet(&CConfigServerLinks::new(
                &VarInt(LINKS.len() as i32),
                &LINKS,
            ))
            .await;
        }

        // known data packs
        self.send_packet(&CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: "1.21",
        }]))
        .await;
        log::debug!("login acknowledged");
    }
}
