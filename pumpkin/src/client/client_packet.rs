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

use super::{authentication::auth_digest, Client, EncryptionError, PlayerConfig};

/// Processes incoming Packets from the Client to the Server
/// Implements the `Client` Packets, So everything before the Play state, then will use the `PlayerPacketProcessor`
pub trait ClientPacketProcessor {
    // Handshake
    fn handle_handshake(&mut self, server: &mut Server, handshake: SHandShake);
    // Status
    fn handle_status_request(&mut self, server: &mut Server, status_request: SStatusRequest);
    fn handle_ping_request(&mut self, server: &mut Server, ping_request: SPingRequest);
    // Login
    fn handle_login_start(&mut self, server: &mut Server, login_start: SLoginStart);
    fn handle_encryption_response(
        &mut self,
        server: &mut Server,
        encryption_response: SEncryptionResponse,
    );
    fn handle_plugin_response(
        &mut self,
        server: &mut Server,
        plugin_response: SLoginPluginResponse,
    );
    fn handle_login_acknowledged(
        &mut self,
        server: &mut Server,
        login_acknowledged: SLoginAcknowledged,
    );
    // Config
    fn handle_client_information(
        &mut self,
        server: &mut Server,
        client_information: SClientInformation,
    );
    fn handle_plugin_message(&mut self, server: &mut Server, plugin_message: SPluginMessage);
    fn handle_known_packs(&mut self, server: &mut Server, config_acknowledged: SKnownPacks);
    fn handle_config_acknowledged(
        &mut self,
        server: &mut Server,
        config_acknowledged: SAcknowledgeFinishConfig,
    );
}

impl ClientPacketProcessor for Client {
    fn handle_handshake(&mut self, _server: &mut Server, handshake: SHandShake) {
        // TODO set protocol version and check protocol version
        self.connection_state = handshake.next_state;
        dbg!("handshake");
    }

    fn handle_status_request(&mut self, server: &mut Server, _status_request: SStatusRequest) {
        dbg!("sending status");

        self.send_packet(CStatusResponse::new(&server.status_response_json))
            .unwrap_or_else(|e| self.kick(&e.to_string()));
    }

    fn handle_ping_request(&mut self, _server: &mut Server, ping_request: SPingRequest) {
        dbg!("ping");
        self.send_packet(CPingResponse::new(ping_request.payload))
            .unwrap_or_else(|e| self.kick(&e.to_string()));
        self.close();
    }

    fn handle_login_start(&mut self, server: &mut Server, login_start: SLoginStart) {
        dbg!("login start");
        // default game profile, when no online mode
        self.gameprofile = Some(GameProfile {
            id: login_start.uuid,
            name: login_start.name,
            properties: vec![],
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
        self.send_packet(packet)
            .unwrap_or_else(|e| self.kick(&e.to_string()));
    }

    fn handle_encryption_response(
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
                Ok(p) => self.gameprofile = Some(p),
                Err(e) => self.kick(&e.to_string()),
            }
        }

        if let Some(profile) = self.gameprofile.as_ref().cloned() {
            let packet = CLoginSuccess::new(profile.id, profile.name, &profile.properties, false);
            self.send_packet(packet)
                .unwrap_or_else(|e| self.kick(&e.to_string()));
        } else {
            self.kick("game profile is none");
        }
    }

    fn handle_plugin_response(
        &mut self,
        _server: &mut Server,
        _plugin_response: SLoginPluginResponse,
    ) {
    }

    fn handle_login_acknowledged(
        &mut self,
        server: &mut Server,
        _login_acknowledged: SLoginAcknowledged,
    ) {
        self.connection_state = ConnectionState::Config;
        server
            .send_brand(self)
            .unwrap_or_else(|e| self.kick(&e.to_string()));
        // known data packs
        self.send_packet(CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: "1.21",
        }]))
        .unwrap_or_else(|e| self.kick(&e.to_string()));
        dbg!("login achnowlaged");
    }
    fn handle_client_information(
        &mut self,
        _server: &mut Server,
        client_information: SClientInformation,
    ) {
        self.config = Some(PlayerConfig {
            locale: client_information.locale,
            view_distance: client_information.view_distance,
            chat_mode: ChatMode::from_varint(client_information.chat_mode),
            chat_colors: client_information.chat_colors,
            skin_parts: client_information.skin_parts,
            main_hand: Hand::from_varint(client_information.main_hand),
            text_filtering: client_information.text_filtering,
            server_listing: client_information.server_listing,
        });
    }

    fn handle_plugin_message(&mut self, _server: &mut Server, plugin_message: SPluginMessage) {
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

    fn handle_known_packs(&mut self, _server: &mut Server, _config_acknowledged: SKnownPacks) {
        for registry in Registry::get_static() {
            self.send_packet(CRegistryData::new(
                &registry.registry_id,
                &registry.registry_entries,
            ))
            .unwrap_or_else(|e| self.kick(&e.to_string()));
        }

        // We are done with configuring
        dbg!("finish config");
        self.send_packet(CFinishConfig::new())
            .unwrap_or_else(|e| self.kick(&e.to_string()));
    }

    fn handle_config_acknowledged(
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
