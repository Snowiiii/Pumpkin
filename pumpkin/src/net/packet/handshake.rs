use std::num::NonZeroI32;

use pumpkin_protocol::{server::handshake::SHandShake, ConnectionState, CURRENT_MC_PROTOCOL};

use crate::{net::Client, server::CURRENT_MC_VERSION};

impl Client {
    pub async fn handle_handshake(&self, handshake: SHandShake) {
        let version = handshake.protocol_version.0;
        self.protocol_version
            .store(version, std::sync::atomic::Ordering::Relaxed);
        *self.server_address.lock().await = handshake.server_address;

        log::debug!("Handshake: next state {:?}", &handshake.next_state);
        self.connection_state.store(handshake.next_state);
        if self.connection_state.load() != ConnectionState::Status {
            let protocol = version;
            match protocol.cmp(&NonZeroI32::from(CURRENT_MC_PROTOCOL).get()) {
                std::cmp::Ordering::Less => {
                    self.kick(&format!("Client outdated ({protocol}), Server uses Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL}")).await;
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {
                    self.kick(&format!("Server outdated, Server uses Minecraft {CURRENT_MC_VERSION}, Protocol {CURRENT_MC_PROTOCOL}")).await;
                }
            }
        }
    }
}
