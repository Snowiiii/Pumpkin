use std::sync::Arc;

use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::{
    math::{get_section_cord, position::WorldPosition, vector3::Vector3},
    GameMode,
};
use pumpkin_protocol::client::play::{CCenterChunk, CUnloadChunk};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

use crate::entity::player::Player;

pub async fn get_view_distance(player: &Player) -> u8 {
    player
        .config
        .lock()
        .await
        .view_distance
        .clamp(2, BASIC_CONFIG.view_distance)
}

pub async fn player_join(player: &Arc<Player>) {
    let chunk_pos = player.living_entity.entity.chunk_pos.load();

    log::debug!("Sending center chunk to {}", player.gameprofile.name);
    player
        .client
        .send_packet(&CCenterChunk {
            chunk_x: chunk_pos.x.into(),
            chunk_z: chunk_pos.z.into(),
        })
        .await;
    let view_distance = get_view_distance(player).await;
    log::debug!(
        "Player {} ({}) joined with view distance: {}",
        player.gameprofile.name,
        player.client.id,
        view_distance
    );

    update_position(player).await;
}

pub async fn update_position(player: &Arc<Player>) {
    if !player.abilities.lock().await.flying {
        player
            .living_entity
            .update_fall_distance(player.gamemode.load() == GameMode::Creative)
            .await;
    }

    let entity = &player.living_entity.entity;

    let view_distance = get_view_distance(player).await;
    let new_chunk_center = entity.chunk_pos.load();

    let old_cylindrical = player.watched_section.load();
    let new_cylindrical = Cylindrical::new(new_chunk_center, view_distance);

    if old_cylindrical != new_cylindrical {
        player
            .client
            .send_packet(&CCenterChunk {
                chunk_x: new_chunk_center.x.into(),
                chunk_z: new_chunk_center.z.into(),
            })
            .await;

        let mut loading_chunks = Vec::new();
        let mut unloading_chunks = Vec::new();
        Cylindrical::for_each_changed_chunk(
            old_cylindrical,
            new_cylindrical,
            |chunk_pos| {
                loading_chunks.push(chunk_pos);
            },
            |chunk_pos| {
                unloading_chunks.push(chunk_pos);
            },
        );

        // Make sure the watched section and the chunk watcher updates are async atomic. We want to
        // ensure what we unload when the player disconnects is correct
        entity.world.mark_chunks_as_watched(&loading_chunks);
        let chunks_to_clean = entity.world.mark_chunks_as_not_watched(&unloading_chunks);
        player.watched_section.store(new_cylindrical);

        if !chunks_to_clean.is_empty() {
            entity.world.clean_chunks(&chunks_to_clean);

            // This can take a little if we are sending a bunch of packets, queue it up :p
            let client = player.client.clone();
            tokio::spawn(async move {
                for chunk in unloading_chunks {
                    if client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                        // We will never un-close a connection
                        break;
                    }
                    client
                        .send_packet(&CUnloadChunk::new(chunk.x, chunk.z))
                        .await;
                }
            });
        }

        if !loading_chunks.is_empty() {
            entity
                .world
                .spawn_world_chunks(player.clone(), loading_chunks, new_chunk_center);
        }
    }
}

#[must_use]
pub const fn chunk_section_from_pos(block_pos: &WorldPosition) -> Vector3<i32> {
    let block_pos = block_pos.0;
    Vector3::new(
        get_section_cord(block_pos.x),
        get_section_cord(block_pos.y),
        get_section_cord(block_pos.z),
    )
}
