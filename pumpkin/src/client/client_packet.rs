use crate::protocol::{
    client::status::{CPingResponse, CStatusResponse},
    server::{
        handshake::SHandShake,
        status::{SPingRequest, SStatusRequest},
    },
    StatusResponse, Version,
};

use super::Client;

pub trait ClientPacketProcessor {
    // Handshake
    fn handle_handshake(&mut self, handshake: SHandShake);
    // Status
    fn handle_status_request(&mut self, status_request: SStatusRequest);
    fn handle_ping_request(&mut self, ping_request: SPingRequest);
}

impl ClientPacketProcessor for Client {
    fn handle_handshake(&mut self, handshake: SHandShake) {
        // TODO set protocol version and check protocol version
        self.connection_state = handshake.next_state;
        dbg!("handshake");
    }

    fn handle_status_request(&mut self, _status_request: SStatusRequest) {
        dbg!("sending status");
        let response = StatusResponse {
            version: Version {
                version: "1.21".into(),
                protocol: 767,
            },
            description: "Pumpkin Server".into(),
        };
        self.send_packet(CStatusResponse::new(
            serde_json::to_string(&response).unwrap(),
        ))
    }

    fn handle_ping_request(&mut self, ping_request: SPingRequest) {
        dbg!("ping");
        self.send_packet(CPingResponse::new(ping_request.payload));
        self.close();
    }
}
