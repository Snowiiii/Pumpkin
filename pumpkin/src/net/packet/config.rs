use std::num::NonZeroU8;

use crate::{
    entity::player::{ChatMode, Hand},
    net::{Client, PlayerConfig},
    server::Server,
};
use core::str;
use num_traits::FromPrimitive;
use pumpkin_protocol::{
    client::config::{CFinishConfig, CRegistryData},
    codec::var_int::VarInt,
    server::config::{
        SClientInformationConfig, SConfigCookieResponse, SKnownPacks, SPluginMessage,
    },
    ConnectionState,
};

impl Client {
    pub async fn handle_client_information_config(
        &self,
        client_information: SClientInformationConfig,
    ) {
        log::debug!("Handling client settings");
        if client_information.view_distance <= 0 {
            self.kick("Cannot have zero or negative view distance!")
                .await;
            return;
        }

        if let (Some(main_hand), Some(chat_mode)) = (
            Hand::from_i32(client_information.main_hand.into()),
            ChatMode::from_i32(client_information.chat_mode.into()),
        ) {
            *self.config.lock().await = Some(PlayerConfig {
                locale: client_information.locale,
                view_distance: unsafe {
                    NonZeroU8::new_unchecked(client_information.view_distance as u8)
                },
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            });
        } else {
            self.kick("Invalid hand or chat type").await;
        }
    }

    pub async fn handle_plugin_message(&self, plugin_message: SPluginMessage) {
        log::debug!("Handling plugin message");
        if plugin_message
            .channel
            .to_string()
            .starts_with("minecraft:brand")
        {
            log::debug!("got a client brand");
            match str::from_utf8(&plugin_message.data) {
                Ok(brand) => *self.brand.lock().await = Some(brand.to_string()),
                Err(e) => self.kick(&e.to_string()).await,
            }
        }
    }

    pub fn handle_config_cookie_response(&self, packet: SConfigCookieResponse) {
        // TODO: allow plugins to access this
        log::debug!(
            "Received cookie_response[config]: key: \"{}\", has_payload: \"{}\", payload_length: \"{}\"",
            packet.key.to_string(),
            packet.has_payload,
            packet.payload_length.unwrap_or(VarInt::from(0)).0
        );
    }

    pub async fn handle_known_packs(&self, server: &Server, _config_acknowledged: SKnownPacks) {
        log::debug!("Handling known packs");
        for registry in &server.cached_registry {
            self.send_packet(&CRegistryData::new(
                &registry.registry_id,
                &registry.registry_entries,
            ))
            .await;
        }

        // We are done with configuring
        log::debug!("finished config");
        self.send_packet(&CFinishConfig::new()).await;
    }

    pub fn handle_config_acknowledged(&self) {
        log::debug!("Handling config acknowledge");
        self.connection_state.store(ConnectionState::Play);
        self.make_player
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
