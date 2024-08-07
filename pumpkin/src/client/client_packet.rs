use num_traits::FromPrimitive;
use pumpkin_protocol::{
    client::{
        config::{CFinishConfig, CKnownPacks, CRegistryData},
        login::{CEncryptionRequest, CLoginSuccess},
        status::{CPingResponse, CStatusResponse},
    },
    server::{
        config::{SAcknowledgeFinishConfig, SClientInformation, SKnownPacks, SPluginMessage},
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
        status::{SPingRequest, SStatusRequest},
    },
    ConnectionState, KnownPack,
};
use pumpkin_registry::Registry;
use rsa::Pkcs1v15Encrypt;
use sha1::{Digest, Sha1};

use crate::{
    client::authentication::{self, GameProfile},
    entity::player::{ChatMode, Hand},
    server::Server,
};

use super::{
    authentication::{auth_digest, unpack_textures},
    Client, EncryptionError, PlayerConfig,
};

/// Processes incoming Packets from the Client to the Server
/// Implements the `Client` Packets
impl Client {
    pub fn handle_handshake(&mut self, _server: &mut Server, handshake: SHandShake) {
        // TODO set protocol version and check protocol version
        self.connection_state = handshake.next_state;
        dbg!("handshake");
    }

    pub fn handle_status_request(&mut self, server: &mut Server, _status_request: SStatusRequest) {
        self.send_packet(CStatusResponse::new(&server.status_response_json));
    }

    pub fn handle_ping_request(&mut self, _server: &mut Server, ping_request: SPingRequest) {
        dbg!("ping");
        self.send_packet(CPingResponse::new(ping_request.payload));
        self.close();
    }

    pub fn handle_login_start(&mut self, server: &mut Server, login_start: SLoginStart) {
        // todo: do basic name validation
        dbg!("login start");
        // default game profile, when no online mode
        // todo: make offline uuid
        self.gameprofile = Some(GameProfile {
            id: login_start.uuid,
            name: login_start.name,
            properties: vec![],
            profile_actions: None,
        });

        // todo: check config for encryption
        let verify_token: [u8; 4] = rand::random();
        let public_key_der = &server.public_key_der;
        let packet = CEncryptionRequest::new(
            "",
            public_key_der,
            &verify_token,
            server.base_config.online_mode, // TODO
        );
        self.send_packet(packet);
    }

    pub fn handle_encryption_response(
        &mut self,
        server: &mut Server,
        encryption_response: SEncryptionResponse,
    ) {
        let shared_secret = server
            .private_key
            .decrypt(Pkcs1v15Encrypt, &encryption_response.shared_secret)
            .map_err(|_| EncryptionError::FailedDecrypt)
            .unwrap();
        self.enable_encryption(&shared_secret)
            .unwrap_or_else(|e| self.kick(&e.to_string()));

        if server.base_config.online_mode {
            let hash = Sha1::new()
                .chain_update(&shared_secret)
                .chain_update(&server.public_key_der)
                .finalize();
            let hash = auth_digest(&hash);
            let ip = self.address.ip();
            match authentication::authenticate(
                &self.gameprofile.as_ref().unwrap().name,
                &hash,
                &ip,
                server,
            ) {
                Ok(p) => {
                    // Check if player should join
                    if let Some(p) = &p.profile_actions {
                        if !server
                            .advanced_config
                            .authentication
                            .player_profile
                            .allow_banned_players
                        {
                            if !p.is_empty() {
                                self.kick("Your account can't join");
                            }
                        } else {
                            for allowed in server
                                .advanced_config
                                .authentication
                                .player_profile
                                .allowed_actions
                                .clone()
                            {
                                if !p.contains(&allowed) {
                                    self.kick("Your account can't join");
                                }
                            }
                        }
                    }
                    self.gameprofile = Some(p);
                }
                Err(e) => self.kick(&e.to_string()),
            }
        }
        for ele in self.gameprofile.as_ref().unwrap().properties.clone() {
            unpack_textures(ele, &server.advanced_config.authentication.textures);
        }

        if let Some(profile) = self.gameprofile.as_ref().cloned() {
            let packet = CLoginSuccess::new(profile.id, profile.name, &profile.properties, false);
            self.send_packet(packet);
        } else {
            self.kick("game profile is none");
        }
    }

    pub fn handle_plugin_response(
        &mut self,
        _server: &mut Server,
        _plugin_response: SLoginPluginResponse,
    ) {
    }

    pub fn handle_login_acknowledged(
        &mut self,
        server: &mut Server,
        _login_acknowledged: SLoginAcknowledged,
    ) {
        self.connection_state = ConnectionState::Config;
        server.send_brand(self);
        // known data packs
        self.send_packet(CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: "1.21",
        }]));
        dbg!("login achnowlaged");
    }
    pub fn handle_client_information(
        &mut self,
        _server: &mut Server,
        client_information: SClientInformation,
    ) {
        self.config = Some(PlayerConfig {
            locale: client_information.locale,
            view_distance: client_information.view_distance,
            chat_mode: ChatMode::from_i32(client_information.chat_mode.into()).unwrap(),
            chat_colors: client_information.chat_colors,
            skin_parts: client_information.skin_parts,
            main_hand: Hand::from_i32(client_information.main_hand.into()).unwrap(),
            text_filtering: client_information.text_filtering,
            server_listing: client_information.server_listing,
        });
    }

    pub fn handle_plugin_message(&mut self, _server: &mut Server, plugin_message: SPluginMessage) {
        if plugin_message.channel.starts_with("minecraft:brand")
            || plugin_message.channel.starts_with("MC|Brand")
        {
            dbg!("got a client brand");
            match String::from_utf8(plugin_message.data) {
                Ok(brand) => self.brand = Some(brand),
                Err(e) => self.kick(&e.to_string()),
            }
        }
    }

    pub fn handle_known_packs(&mut self, _server: &mut Server, _config_acknowledged: SKnownPacks) {
        for registry in Registry::get_static() {
            self.send_packet(CRegistryData::new(
                &registry.registry_id,
                &registry.registry_entries,
            ));
        }

        // We are done with configuring
        dbg!("finish config");
        self.send_packet(CFinishConfig::new());
    }

    pub fn handle_config_acknowledged(
        &mut self,
        server: &mut Server,
        _config_acknowledged: SAcknowledgeFinishConfig,
    ) {
        dbg!("config acknowledged");
        self.connection_state = ConnectionState::Play;
        // generate a player
        server.spawn_player(self);
    }
}
