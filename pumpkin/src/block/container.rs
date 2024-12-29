use crate::entity::player::Player;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_inventory::Container;
use pumpkin_world::block::block_registry::Block;

#[async_trait]
pub trait ContainerBlock<C: Container> {
    const UNIQUE: bool;
    async fn open(&self, block: &Block, player: &Player, location: WorldPosition, server: &Server)
    where
        C: Default + 'static,
    {
        if Self::UNIQUE {
            let mut open_containers = server.open_containers.write().await;
            let id = open_containers.new_unique::<C>(Some(block.clone()), player.gameprofile.id);
            let window_type = open_containers
                .containers_by_id
                .get(&id)
                .expect("just created it")
                .window_type()
                .await;
            drop(open_containers);
            player.open_container.store(Some(id));
            player.open_container(server, window_type).await;
        } else {
            let player_id = player.gameprofile.id;
            let mut open_containers = server.open_containers.write().await;
            let opened_container = open_containers.get_mut_by_location(&location);

            #[allow(clippy::option_if_let_else)]
            let container = match opened_container {
                Some(container) => container,
                None => open_containers
                    .new_by_location::<C>(location, Some(block.clone()))
                    .unwrap(),
            };

            container.add_player(player_id);
            player.open_container.store(Some(container.id));
            player
                .open_container(server, container.window_type().await)
                .await;
        }
    }

    async fn close(&self, location: WorldPosition, server: &Server, player: &Player)
    where
        C: Default + 'static,
    {
        if Self::UNIQUE {
            self.destroy(location, server, player).await;
        } else {
            let mut containers = server.open_containers.write().await;
            if let Some(container) = containers.get_mut_by_location(&location) {
                container.remove_player(player.gameprofile.id);
            }
        }
    }

    /// The standard destroy with container removes the player forcibly from the container,
    /// drops items to the floor, and back to the player's inventory if the item stack is in movement.
    async fn destroy(&self, location: WorldPosition, server: &Server, player: &Player) {
        let mut open_containers = server.open_containers.write().await;

        let mut inventory = player.inventory().lock().await;
        let mut carried_item = player.carried_item.load();
        let player_ids = open_containers
            .destroy_by_location(&location, &mut inventory, &mut carried_item)
            .await;
        player.carried_item.store(carried_item);
        for player_id in player_ids {
            if let Some(player) = server.get_player_by_uuid(player_id).await {
                player.open_container.store(None);
            }
        }
    }
}
