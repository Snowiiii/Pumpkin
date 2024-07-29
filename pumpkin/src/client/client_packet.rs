
use crate::protocol::{
    client::{
        login::{CEncryptionRequest, CLoginSuccess},
        status::{CPingResponse, CStatusResponse},
    },
    server::{
        handshake::SHandShake,
        login::{SEncryptionResponse, SLoginAcknowledged, SLoginPluginResponse, SLoginStart},
        status::{SPingRequest, SStatusRequest},
    },
    ConnectionState,
};

use super::Client;

pub trait ClientPacketProcessor {
    // Handshake
    fn handle_handshake(&mut self, handshake: SHandShake);
    // Status
    fn handle_status_request(&mut self, status_request: SStatusRequest);
    fn handle_ping_request(&mut self, ping_request: SPingRequest);
    // Login
    fn handle_login_start(&mut self, login_start: SLoginStart);
    fn handle_encryption_response(&mut self, encryption_response: SEncryptionResponse);
    fn handle_plugin_response(&mut self, plugin_response: SLoginPluginResponse);
    fn handle_login_acknowledged(&mut self, login_acknowledged: SLoginAcknowledged);
}

impl ClientPacketProcessor for Client {
    fn handle_handshake(&mut self, handshake: SHandShake) {
        // TODO set protocol version and check protocol version
        self.connection_state = handshake.next_state;
        dbg!("handshake");
    }

    fn handle_status_request(&mut self, _status_request: SStatusRequest) {
        dbg!("sending status");

        self.send_packet(CStatusResponse::new(
            serde_json::to_string(&self.server.status_response).unwrap(),
        ))
    }

    fn handle_ping_request(&mut self, ping_request: SPingRequest) {
        dbg!("ping");
        self.send_packet(CPingResponse::new(ping_request.payload));
        self.close();
    }

    fn handle_login_start(&mut self, login_start: SLoginStart) {
        dbg!("login start");
        self.name = Some(login_start.name);
        self.uuid = Some(login_start.uuid);
        let verify_token: [u8; 4] = rand::random();
        let public_key_der = &self.server.to_owned().public_key_der;
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

    fn handle_encryption_response(&mut self, encryption_response: SEncryptionResponse) {
        dbg!("encryption response");
        // should be impossible
        if self.uuid.is_none() || self.name.is_none() {
            self.kick("UUID or Name is none".into());
            return;
        }
        self.enable_encryption(encryption_response.shared_secret)
            .unwrap();

        let packet = CLoginSuccess::new(self.uuid.unwrap(), self.name.clone().unwrap(), 0, false);
        self.send_packet(packet);
    }

    fn handle_plugin_response(&mut self, plugin_response: SLoginPluginResponse) {}

    fn handle_login_acknowledged(&mut self, login_acknowledged: SLoginAcknowledged) {
        self.connection_state = ConnectionState::Config;
        dbg!("login achnowlaged");
    }
}
