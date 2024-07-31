use crate::{
    protocol::{
        client::{
            config::{CFinishConfig, CKnownPacks, CPluginMessage, CRegistryData, Entry},
            login::{CEncryptionRequest, CLoginSuccess},
            status::{CPingResponse, CStatusResponse},
        },
        server::{
            config::{SAcknowledgeFinishConfig, SClientInformation, SKnownPacks},
            handshake::SHandShake,
            login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
            status::{SPingRequest, SStatusRequest},
        },
        ConnectionState, KnownPack,
    },
    server::Server,
};

use super::{Client, PlayerConfig};

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

        let response = serde_json::to_string(&server.status_response).unwrap();

        self.send_packet(CStatusResponse::new(response));
    }

    fn handle_ping_request(&mut self, _server: &mut Server, ping_request: SPingRequest) {
        dbg!("ping");
        self.send_packet(CPingResponse::new(ping_request.payload));
        self.close();
    }

    fn handle_login_start(&mut self, server: &mut Server, login_start: SLoginStart) {
        dbg!("login start");
        self.name = Some(login_start.name);
        self.uuid = Some(login_start.uuid);
        let verify_token: [u8; 4] = rand::random();
        let public_key_der = &server.public_key_der;
        let packet = CEncryptionRequest::new(
            "".into(),
            public_key_der.len() as i32,
            public_key_der,
            verify_token.len() as i32,
            &verify_token,
            false, // TODO
        );
        self.send_packet(packet);
    }

    fn handle_encryption_response(
        &mut self,
        server: &mut Server,
        encryption_response: SEncryptionResponse,
    ) {
        dbg!("encryption response");
        // should be impossible
        if self.uuid.is_none() || self.name.is_none() {
            self.kick("UUID or Name is none".into());
            return;
        }
        self.enable_encryption(server, encryption_response.shared_secret)
            .unwrap();

        let packet = CLoginSuccess::new(self.uuid.unwrap(), self.name.clone().unwrap(), 0, false);
        self.send_packet(packet);
    }

    fn handle_plugin_response(
        &mut self,
        _server: &mut Server,
        _plugin_response: SLoginPluginResponse,
    ) {
    }

    fn handle_login_acknowledged(
        &mut self,
        _server: &mut Server,
        _login_acknowledged: SLoginAcknowledged,
    ) {
        self.connection_state = ConnectionState::Config;
        Server::send_brand(self);
        // known data packs
        self.send_packet(CKnownPacks::new(
            1,
            &[KnownPack {
                namespace: "minecraft".to_string(),
                id: "core".to_string(),
                version: "1.21".to_string(),
            }],
        ));
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
            chat_mode: client_information.chat_mode,
            chat_colors: client_information.chat_colors,
            skin_parts: client_information.skin_parts,
            main_hand: client_information.main_hand,
            text_filtering: client_information.text_filtering,
            server_listing: client_information.server_listing,
        });
    }

    fn handle_known_packs(&mut self, server: &mut Server, config_acknowledged: SKnownPacks) {
        self.send_packet(CRegistryData::new(
            "0".into(),
            1,
            vec![
                Entry {
                    entry_id: "minecraft:dimension_type".into(),
                    has_data: true,
                },
                /*    Entry {
                    entry_id: "minecraft:worldgen/biome".into(),
                    has_data: true,
                },
                Entry {
                    entry_id: "minecraft:chat_type".into(),
                    has_data: true,
                },
                Entry {
                    entry_id: "minecraft:damage_type".into(),
                    has_data: true,
                    }, */
            ],
        ));

        // We are done with configuring
        dbg!("finish config");
        self.send_packet(CFinishConfig::new());
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
