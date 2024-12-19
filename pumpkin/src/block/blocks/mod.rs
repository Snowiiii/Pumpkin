use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::{Container, OpenContainer, WindowType};
use pumpkin_world::block::block_registry::Block;

use crate::{entity::player::Player, server::Server};

pub(crate) mod chest;
pub(crate) mod crafting_table;
pub(crate) mod furnace;
pub(crate) mod jukebox;

/// The standard destroy with container removes the player forcibly from the container,
/// drops items to the floor and back to the player's inventory if the item stack is in movement.
pub async fn standard_on_destroy_with_container(
    block: &Block,
    player: &Player,
    location: WorldPosition,
    server: &Server,
) {
    // TODO: drop all items and back to players inventory if in motion
    let entity_id = player.entity_id();
    if let Some(container_id) = server.get_container_id(location, block.clone()).await {
        let mut open_containers = server.open_containers.write().await;
        if let Some(container) = open_containers.get_mut(&u64::from(container_id)) {
            log::info!("Good ct ID: {}", container_id);
            container.remove_player(entity_id);
            container.clear_all_slots().await;
            player.open_container.store(None);
            close_all_in_container(player, container).await;
        }
    }
}

/// The standard open container creates a new container if a container of the same block
/// type does not exist at the selected block location. If a container of the same type exists, the player
/// is added to the currently connected players to that container.  
pub async fn standard_open_container<C: Container + Default + 'static>(
    block: &Block,
    player: &Player,
    location: WorldPosition,
    server: &Server,
    window_type: WindowType,
) {
    let entity_id = player.entity_id();
    if let Some(container_id) = server.get_container_id(location, block.clone()).await {
        let mut open_containers = server.open_containers.write().await;
        if let Some(container) = open_containers.get_mut(&u64::from(container_id)) {
            log::info!("Good ID: {}", container_id);
            container.add_player(entity_id);
            player.open_container.store(Some(container_id.into()));
        }
    } else {
        let mut open_containers = server.open_containers.write().await;

        let new_id = server.new_container_id();
        log::info!("New ID: {}", new_id);

        let open_container =
            OpenContainer::new_empty_container::<C>(entity_id, Some(location), Some(block.clone()));
        open_containers.insert(new_id.into(), open_container);
        player.open_container.store(Some(new_id.into()));
    }
    player.open_container(server, window_type).await;
}

pub async fn close_all_in_container(player: &Player, container: &OpenContainer) {
    for id in container.all_player_ids() {
        if let Some(y) = player.world().get_player_by_entityid(id).await {
            y.close_container().await;
        }
    }
}
