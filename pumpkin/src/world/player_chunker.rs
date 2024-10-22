use std::sync::Arc;

use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
};
use pumpkin_protocol::client::play::{CCenterChunk, CUnloadChunk};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

use crate::entity::player::Player;

use super::World;

pub async fn get_view_distance(player: &Player) -> i8 {
    player
        .config
        .lock()
        .await
        .view_distance
        .clamp(2, BASIC_CONFIG.view_distance as i8)
}

pub async fn player_join(world: &World, player: Arc<Player>) {
    let new_watched = chunk_section_from_pos(&player.living_entity.entity.block_pos.load());
    player.watched_section.store(new_watched);
    let chunk_pos = player.living_entity.entity.chunk_pos.load();

    assert_eq!(new_watched.x, chunk_pos.x);
    assert_eq!(new_watched.z, chunk_pos.z);

    log::debug!("Sending center chunk to {}", player.client.id);
    player
        .client
        .send_packet(&CCenterChunk {
            chunk_x: chunk_pos.x.into(),
            chunk_z: chunk_pos.z.into(),
        })
        .await;
    let view_distance = i32::from(get_view_distance(&player).await);
    log::debug!(
        "Player {} ({}) joined with view distance: {}",
        player.gameprofile.name,
        player.client.id,
        view_distance
    );

    let new_cylindrical = Cylindrical::new(Vector2::new(chunk_pos.x, chunk_pos.z), view_distance);
    let loading_chunks = new_cylindrical.all_chunks_within();

    world.mark_chunks_as_watched(&loading_chunks).await;
    world.spawn_world_chunks(player.client.clone(), loading_chunks);
}

pub async fn update_position(player: &Player) {
    let entity = &player.living_entity.entity;
    let current_watched = player.watched_section.load();
    let new_watched = chunk_section_from_pos(&entity.block_pos.load());
    player.watched_section.store(new_watched);

    if current_watched != new_watched {
        //log::debug!("changing chunks");
        let chunk_pos = entity.chunk_pos.load();
        assert_eq!(new_watched.x, chunk_pos.x);
        assert_eq!(new_watched.z, chunk_pos.z);

        player
            .client
            .send_packet(&CCenterChunk {
                chunk_x: chunk_pos.x.into(),
                chunk_z: chunk_pos.z.into(),
            })
            .await;

        let view_distance = i32::from(get_view_distance(player).await);
        let old_cylindrical = Cylindrical::new(
            Vector2::new(current_watched.x, current_watched.z),
            view_distance,
        );
        let new_cylindrical = Cylindrical::new(chunk_pos, view_distance);

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
        if !loading_chunks.is_empty() {
            entity.world.mark_chunks_as_watched(&loading_chunks).await;
            entity
                .world
                .spawn_world_chunks(player.client.clone(), loading_chunks);
        }

        if !unloading_chunks.is_empty() {
            entity
                .world
                .mark_chunks_as_not_watched(&unloading_chunks)
                .await;
            // we may don't need to iter twice
            for chunk in unloading_chunks {
                if !player
                    .client
                    .closed
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    player
                        .client
                        .send_packet(&CUnloadChunk::new(chunk.x, chunk.z))
                        .await;
                }
            }
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
