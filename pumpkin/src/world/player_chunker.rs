use std::sync::Arc;

use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
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
    player.watched_section.store(new_watched);
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

    world.spawn_world_chunks(player, &loading_chunks);
}

pub async fn update_position(player: &Arc<Player>) {
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

        let view_distance = get_view_distance(player).await;
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
            //let inst = std::time::Instant::now();
            entity
                .world
                .spawn_world_chunks(player.clone(), &loading_chunks);
            //log::debug!("Loading chunks took {:?}", inst.elapsed());
        }

        if !unloading_chunks.is_empty() {
            // We want to check if this chunk is still pending
            // if it is -> ignore

            //let inst = std::time::Instant::now();

            let watched_chunks: Vec<_> = {
                let mut pending_chunks = player.pending_chunks.lock();
                unloading_chunks
                    .into_iter()
                    .filter(|chunk| {
                        if let Some(handles) = pending_chunks.get_mut(chunk) {
                            if let Some((count, handle)) = handles
                                .iter_mut()
                                .rev()
                                .enumerate()
                                .find(|(_, handle)| !handle.aborted())
                            {
                                log::debug!("Aborting chunk {:?} ({}) (unload)", chunk, count);
                                // We want to abort the last queued chunk, that we if a client still
                                // has a pending request for this chunk, we dont need to do the work
                                // twice
                                handle.abort();
                            } else {
                                log::warn!(
                                    "Aborting chunk {:?} but all were already aborted!",
                                    chunk
                                );
                            }
                            false
                        } else {
                            true
                        }
                    })
                    .collect()
            };

            //log::debug!("Unloading chunks took {:?} (1)", inst.elapsed());
            let chunks_to_clean = entity.world.mark_chunks_as_not_watched(&watched_chunks);
            entity.world.clean_chunks(&chunks_to_clean);

            //log::debug!("Unloading chunks took {:?} (2)", inst.elapsed());
            // This can take a little if we are sending a bunch of packets, queue it up :p
            let client = player.client.clone();
            tokio::spawn(async move {
                for chunk in watched_chunks {
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
