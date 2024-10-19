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
    let watched_section = new_watched;
    let chunk_pos = player.living_entity.entity.chunk_pos.load();
    player
        .client
        .send_packet(&CCenterChunk {
            chunk_x: chunk_pos.x.into(),
            chunk_z: chunk_pos.z.into(),
        })
        .await;
    let view_distance = get_view_distance(&player).await as i32;
    dbg!(view_distance);
    let old_cylindrical = Cylindrical::new(
        Vector2::new(watched_section.x, watched_section.z),
        view_distance,
    );
    let new_cylindrical = Cylindrical::new(Vector2::new(chunk_pos.x, chunk_pos.z), view_distance);
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
            // player
            //     .client
            //     .send_packet(&CUnloadChunk::new(chunk_pos.x, chunk_pos.z));
        },
        true,
    );
    if !loading_chunks.is_empty() {
        world.mark_chunks_as_watched(&loading_chunks).await;
        world
            .spawn_world_chunks(player.client.clone(), loading_chunks, view_distance)
            .await;
    }

    if !unloading_chunks.is_empty() {
        world.mark_chunks_as_not_watched(&unloading_chunks).await;
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

pub async fn update_position(player: &Player) {
    let entity = &player.living_entity.entity;
    let current_watched = player.watched_section.load();
    let new_watched = chunk_section_from_pos(&entity.block_pos.load());
    if current_watched != new_watched {
        let chunk_pos = entity.chunk_pos.load();
        player
            .client
            .send_packet(&CCenterChunk {
                chunk_x: chunk_pos.x.into(),
                chunk_z: chunk_pos.z.into(),
            })
            .await;

        let view_distance = get_view_distance(player).await as i32;
        let old_cylindrical = Cylindrical::new(
            Vector2::new(current_watched.x, current_watched.z),
            view_distance,
        );
        let new_cylindrical = Cylindrical::new(chunk_pos, view_distance);

        player.watched_section.store(new_watched);

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
                // player
                //     .client
                //     .send_packet(&CUnloadChunk::new(chunk_pos.x, chunk_pos.z));
            },
            false,
        );
        if !loading_chunks.is_empty() {
            entity.world.mark_chunks_as_watched(&loading_chunks).await;
            entity
                .world
                .spawn_world_chunks(player.client.clone(), loading_chunks, view_distance)
                .await;
        }

        if !unloading_chunks.is_empty() {
            entity
                .world
                .mark_chunks_as_not_watched(&unloading_chunks)
                .await;
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

pub const fn chunk_section_from_pos(block_pos: &WorldPosition) -> Vector3<i32> {
    let block_pos = block_pos.0;
    Vector3::new(
        get_section_cord(block_pos.x),
        get_section_cord(block_pos.y),
        get_section_cord(block_pos.z),
    )
}
