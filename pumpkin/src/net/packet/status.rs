use pumpkin_protocol::{client::status::CPingResponse, server::status::SStatusPingRequest};

use crate::{net::Client, server::Server};

impl Client {
    pub async fn handle_status_request(&self, server: &Server) {
        log::debug!("Handling status request");
        let status = server.get_status();
        self.send_packet(&status.lock().await.get_status()).await;
    }

    pub async fn handle_ping_request(&self, ping_request: SStatusPingRequest) {
        log::debug!("Handling ping request");
        self.send_packet(&CPingResponse::new(ping_request.payload))
            .await;
        self.close();
    }
}
