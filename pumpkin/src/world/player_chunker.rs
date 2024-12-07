use std::sync::Arc;

use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::{
    math::{get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3},
    GameMode,
};
use pumpkin_protocol::client::play::{CCenterChunk, CUnloadChunk};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

use crate::entity::player::Player;

use super::World;

pub async fn get_view_distance(player: &Player) -> u8 {
    player
        .config
        .lock()
        .await
        .view_distance
        .clamp(2, BASIC_CONFIG.view_distance)
}

pub async fn player_join(world: &World, player: Arc<Player>) {
    let new_watched = chunk_section_from_pos(&player.living_entity.entity.block_pos.load());

    let mut cylindrical = player.watched_section.load();
    cylindrical.center = new_watched.into();
    player.watched_section.store(cylindrical);

    let chunk_pos = player.living_entity.entity.chunk_pos.load();

    assert_eq!(new_watched.x, chunk_pos.x);
    assert_eq!(new_watched.z, chunk_pos.z);

    log::debug!("Sending center chunk to {}", player.gameprofile.name);
    player
        .client
        .send_packet(&CCenterChunk {
            chunk_x: chunk_pos.x.into(),
            chunk_z: chunk_pos.z.into(),
        })
        .await;
    let view_distance = get_view_distance(&player).await;
    log::debug!(
        "Player {} ({}) joined with view distance: {}",
        player.gameprofile.name,
        player.gameprofile.name,
        view_distance
    );

    let new_cylindrical = Cylindrical::new(Vector2::new(chunk_pos.x, chunk_pos.z), view_distance);
    let loading_chunks = new_cylindrical.all_chunks_within();

    if !loading_chunks.is_empty() {
        world.spawn_world_chunks(player, loading_chunks);
    }
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
    let chunk_center = chunk_section_from_pos(&entity.block_pos.load()).into();

    let old_cylindrical = player.watched_section.load();
    let new_cylindrical = Cylindrical::new(chunk_center, view_distance);

    if old_cylindrical != new_cylindrical {
        player.watched_section.store(new_cylindrical);

        //log::debug!("changing chunks");
        let chunk_pos = entity.chunk_pos.load();
        assert_eq!(new_cylindrical.center.x, chunk_pos.x);
        assert_eq!(new_cylindrical.center.z, chunk_pos.z);

        player
            .client
            .send_packet(&CCenterChunk {
                chunk_x: chunk_pos.x.into(),
                chunk_z: chunk_pos.z.into(),
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
        if !loading_chunks.is_empty() {
            //let inst = std::time::Instant::now();

            // loading_chunks.sort_by(|a, b| {
            //     let distance_a_squared = a.sub(a).length_squared();
            //     let distance_b_squared = b.sub(a).length_squared();
            //     distance_a_squared.cmp(&distance_b_squared)
            // });

            entity
                .world
                .spawn_world_chunks(player.clone(), loading_chunks);
            //log::debug!("Loading chunks took {:?}", inst.elapsed());
        }

        if !unloading_chunks.is_empty() {
            //let inst = std::time::Instant::now();

            //log::debug!("Unloading chunks took {:?} (1)", inst.elapsed());
            let chunks_to_clean = entity.world.mark_chunks_as_not_watched(&unloading_chunks);
            entity.world.clean_chunks(&chunks_to_clean);

            //log::debug!("Unloading chunks took {:?} (2)", inst.elapsed());
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
            //log::debug!("Unloading chunks took {:?} (3)", inst.elapsed());
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
