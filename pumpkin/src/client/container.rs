use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_core::text::TextComponent;
use pumpkin_core::GameMode;
use pumpkin_inventory::container_click::{
    Click, ClickType, KeyClick, MouseClick, MouseDragState, MouseDragType,
};
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::window_property::{WindowProperty, WindowPropertyTrait};
use pumpkin_inventory::{container_click, InventoryError, OptionallyCombinedContainer};
use pumpkin_inventory::{Container, WindowType};
use pumpkin_protocol::client::play::{
    CCloseContainer, COpenScreen, CSetContainerContent, CSetContainerProperty, CSetContainerSlot,
};
use pumpkin_protocol::server::play::SClickContainer;
use pumpkin_protocol::slot::Slot;
use pumpkin_protocol::VarInt;
use pumpkin_world::item::item_registry::Item;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

impl Player {
    pub async fn open_container(&self, server: &Server, window_type: WindowType) {
        let mut inventory = self.inventory().lock().await;
        inventory.state_id = 0;
        inventory.total_opened_containers += 1;
        let mut container = self.get_open_container(server).await;
        let mut container = match container.as_mut() {
            Some(container) => Some(container.lock().await),
            None => None,
        };
        let window_title = container.as_ref().map_or_else(
            || inventory.window_name(),
            |container| container.window_name(),
        );
        let title = TextComponent::text(window_title);

        self.client
            .send_packet(&COpenScreen::new(
                inventory.total_opened_containers.into(),
                VarInt(window_type as i32),
                title,
            ))
            .await;
        drop(inventory);
        self.set_container_content(container.as_deref_mut()).await;
    }

    pub async fn set_container_content(&self, container: Option<&mut Box<dyn Container>>) {
        let mut inventory = self.inventory().lock().await;

        let total_opened_containers = inventory.total_opened_containers;
        let id = if container.is_some() {
            total_opened_containers
        } else {
            0
        };

        let container = OptionallyCombinedContainer::new(&mut inventory, container);

        let slots: Vec<Slot> = container
            .all_slots_ref()
            .into_iter()
            .map(Slot::from)
            .collect();

        let carried_item = self
            .carried_item
            .load()
            .as_ref()
            .map_or_else(Slot::empty, std::convert::Into::into);

        inventory.state_id += 1;
        let packet = CSetContainerContent::new(
            id.into(),
            (inventory.state_id).into(),
            &slots,
            &carried_item,
        );
        self.client.send_packet(&packet).await;
    }

    /// The official Minecraft client is weird, and will always just close *any* window that is opened when this gets sent
    pub async fn close_container(&self) {
        let mut inventory = self.inventory().lock().await;
        inventory.total_opened_containers += 1;
        self.client
            .send_packet(&CCloseContainer::new(
                inventory.total_opened_containers.into(),
            ))
            .await;
    }

    pub async fn set_container_property<T: WindowPropertyTrait>(
        &mut self,
        window_property: WindowProperty<T>,
    ) {
        let (id, value) = window_property.into_tuple();
        self.client
            .send_packet(&CSetContainerProperty::new(
                self.inventory().lock().await.total_opened_containers.into(),
                id,
                value,
            ))
            .await;
    }

    pub async fn handle_click_container(
        &self,
        server: &Arc<Server>,
        packet: SClickContainer,
    ) -> Result<(), InventoryError> {
        let opened_container = self.get_open_container(server).await;
        let mut opened_container = match opened_container.as_ref() {
            Some(container) => Some(container.lock().await),
            None => None,
        };
        let drag_handler = &server.drag_handler;

        let state_id = self.inventory().lock().await.state_id;
        // This is just checking for regular desync, client hasn't done anything malicious
        if state_id != packet.state_id.0 as u32 {
            self.set_container_content(opened_container.as_deref_mut())
                .await;
            return Ok(());
        }

        if opened_container.is_some() {
            let total_containers = self.inventory().lock().await.total_opened_containers;
            if packet.window_id.0 != total_containers {
                return Err(InventoryError::ClosedContainerInteract(self.entity_id()));
            }
        } else if packet.window_id.0 != 0 {
            return Err(InventoryError::ClosedContainerInteract(self.entity_id()));
        }

        let click = Click::new(packet.mode, packet.button, packet.slot)?;
        let (crafted_item, crafted_item_slot) = {
            let mut inventory = self.inventory().lock().await;
            let combined =
                OptionallyCombinedContainer::new(&mut inventory, opened_container.as_deref_mut());
            (
                combined.crafted_item_slot(),
                combined.crafting_output_slot(),
            )
        };
        let crafted_is_picked = crafted_item.is_some()
            && match click.slot {
                container_click::Slot::Normal(slot) => {
                    crafted_item_slot.is_some_and(|crafted_slot| crafted_slot == slot)
                }
                container_click::Slot::OutsideInventory => false,
            };
        let mut update_whole_container = false;

        let click_slot = click.slot;
        self.match_click_behaviour(
            opened_container.as_deref_mut(),
            click,
            drag_handler,
            &mut update_whole_container,
            crafted_is_picked,
        )
        .await?;
        // Checks for if crafted item has been taken
        {
            let mut inventory = self.inventory().lock().await;
            let mut combined =
                OptionallyCombinedContainer::new(&mut inventory, opened_container.as_deref_mut());
            if combined.crafted_item_slot().is_none() && crafted_item.is_some() {
                combined.recipe_used();
            }

            // TODO: `combined.craft` uses rayon! It should be called from `rayon::spawn` and its
            // result passed to the tokio runtime via a channel!
            if combined.craft() {
                drop(inventory);
                self.set_container_content(opened_container.as_deref_mut())
                    .await;
            }
        }

        if let Some(mut opened_container) = opened_container {
            if update_whole_container {
                drop(opened_container);
                self.send_whole_container_change(server).await?;
            } else if let container_click::Slot::Normal(slot_index) = click_slot {
                let mut inventory = self.inventory().lock().await;
                let combined_container =
                    OptionallyCombinedContainer::new(&mut inventory, Some(&mut opened_container));
                if let Some(slot) = combined_container.get_slot_excluding_inventory(slot_index) {
                    let slot = Slot::from(slot);
                    drop(opened_container);
                    self.send_container_changes(server, slot_index, slot)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn match_click_behaviour(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        click: Click,
        drag_handler: &DragHandler,
        update_whole_container: &mut bool,
        using_crafting_slot: bool,
    ) -> Result<(), InventoryError> {
        match click.click_type {
            ClickType::MouseClick(mouse_click) => {
                self.mouse_click(
                    opened_container,
                    mouse_click,
                    click.slot,
                    using_crafting_slot,
                )
                .await
            }
            ClickType::ShiftClick => {
                self.shift_mouse_click(opened_container, click.slot, using_crafting_slot)
                    .await
            }
            ClickType::KeyClick(key_click) => match click.slot {
                container_click::Slot::Normal(slot) => {
                    self.number_button_pressed(
                        opened_container,
                        key_click,
                        slot,
                        using_crafting_slot,
                    )
                    .await
                }
                container_click::Slot::OutsideInventory => Err(InventoryError::InvalidPacket),
            },
            ClickType::CreativePickItem => {
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.creative_pick_item(opened_container, slot).await
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::DoubleClick => {
                *update_whole_container = true;
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.double_click(opened_container, slot).await
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::MouseDrag { drag_state } => {
                if drag_state == MouseDragState::End {
                    *update_whole_container = true;
                }
                self.mouse_drag(drag_handler, opened_container, drag_state)
                    .await
            }
            ClickType::DropType(_drop_type) => {
                log::debug!("todo");
                Ok(())
            }
        }
    }

    async fn mouse_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_click: MouseClick,
        slot: container_click::Slot,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        match slot {
            container_click::Slot::Normal(slot) => {
                let mut carried_item = self.carried_item.load();
                let res = container.handle_item_change(
                    &mut carried_item,
                    slot,
                    mouse_click,
                    taking_crafted,
                );
                self.carried_item.store(carried_item);
                res
            }
            container_click::Slot::OutsideInventory => Ok(()),
        }
    }

    async fn shift_mouse_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: container_click::Slot,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        match slot {
            container_click::Slot::Normal(slot) => {
                let all_slots = container.all_slots();
                if let Some(item_in_pressed_slot) = all_slots[slot].to_owned() {
                    let slots = all_slots.into_iter().enumerate();
                    // Hotbar
                    let find_condition = |(slot_number, slot): (usize, &mut Option<ItemStack>)| {
                        // TODO: Check for max item count here
                        match slot {
                            Some(item) => (item.item_id == item_in_pressed_slot.item_id
                                && item.item_count != 64)
                                .then_some(slot_number),
                            None => Some(slot_number),
                        }
                    };

                    let slots = if slot > 35 {
                        slots.skip(9).find_map(find_condition)
                    } else {
                        slots.skip(36).rev().find_map(find_condition)
                    };
                    if let Some(slot) = slots {
                        let mut item_slot = container.all_slots()[slot].map(|i| i);
                        container.handle_item_change(
                            &mut item_slot,
                            slot,
                            MouseClick::Left,
                            taking_crafted,
                        )?;
                        *container.all_slots()[slot] = item_slot;
                    }
                }
            }
            container_click::Slot::OutsideInventory => (),
        };
        Ok(())
    }

    async fn number_button_pressed(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        key_click: KeyClick,
        slot: usize,
        taking_crafted: bool,
    ) -> Result<(), InventoryError> {
        let changing_slot = match key_click {
            KeyClick::Slot(slot) => slot,
            KeyClick::Offhand => 45,
        };
        let mut inventory = self.inventory().lock().await;
        let mut changing_item_slot = inventory.get_slot(changing_slot as usize)?.to_owned();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        container.handle_item_change(
            &mut changing_item_slot,
            slot,
            MouseClick::Left,
            taking_crafted,
        )?;
        *inventory.get_slot(changing_slot as usize)? = changing_item_slot;
        Ok(())
    }

    async fn creative_pick_item(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: usize,
    ) -> Result<(), InventoryError> {
        if self.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        if let Some(Some(item)) = container.all_slots().get_mut(slot) {
            self.carried_item.store(Some(item.to_owned()));
        }
        Ok(())
    }

    async fn double_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory().lock().await;
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        let mut slots = container.all_slots();

        let Some(item) = slots.get_mut(slot) else {
            return Ok(());
        };
        let Some(mut carried_item) = **item else {
            return Ok(());
        };
        **item = None;

        for slot in slots.iter_mut().filter_map(|slot| slot.as_mut()) {
            if slot.item_id == carried_item.item_id {
                // TODO: Check for max stack size
                if slot.item_count + carried_item.item_count <= 64 {
                    slot.item_count = 0;
                    carried_item.item_count = 64;
                } else {
                    let to_remove = slot.item_count - (64 - carried_item.item_count);
                    slot.item_count -= to_remove;
                    carried_item.item_count += to_remove;
                }

                if carried_item.item_count == 64 {
                    break;
                }
            }
        }
        self.carried_item.store(Some(carried_item));
        Ok(())
    }

    async fn mouse_drag(
        &self,
        drag_handler: &DragHandler,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_drag_state: MouseDragState,
    ) -> Result<(), InventoryError> {
        let player_id = self.entity_id();
        let container_id = opened_container
            .as_ref()
            .map_or(player_id as u64, |container| {
                container.internal_pumpkin_id()
            });
        match mouse_drag_state {
            MouseDragState::Start(drag_type) => {
                if drag_type == MouseDragType::Middle && self.gamemode.load() != GameMode::Creative
                {
                    Err(InventoryError::PermissionError)?;
                }
                drag_handler
                    .new_drag(container_id, player_id, drag_type)
                    .await
            }
            MouseDragState::AddSlot(slot) => {
                drag_handler.add_slot(container_id, player_id, slot).await
            }
            MouseDragState::End => {
                let mut inventory = self.inventory().lock().await;
                let mut container =
                    OptionallyCombinedContainer::new(&mut inventory, opened_container);
                let mut carried_item = self.carried_item.load();
                let res = drag_handler
                    .apply_drag(&mut carried_item, &mut container, &container_id, player_id)
                    .await;
                self.carried_item.store(carried_item);
                res
            }
        }
    }

    async fn get_current_players_in_container(&self, server: &Server) -> Vec<Arc<Self>> {
        let player_ids: Vec<i32> = {
            let open_containers = server.open_containers.read().await;
            open_containers
                .get(&self.open_container.load().unwrap())
                .unwrap()
                .all_player_ids()
                .into_iter()
                .filter(|player_id| *player_id != self.entity_id())
                .collect()
        };
        let player_token = self.gameprofile.id;

        // TODO: Figure out better way to get only the players from player_ids
        // Also refactor out a better method to get individual advanced state ids

        let players = self
            .living_entity
            .entity
            .world
            .current_players
            .lock()
            .await
            .iter()
            .filter_map(|(token, player)| {
                if *token == player_token {
                    None
                } else {
                    let entity_id = player.entity_id();
                    player_ids.contains(&entity_id).then(|| player.clone())
                }
            })
            .collect();
        players
    }

    async fn send_container_changes(
        &self,
        server: &Server,
        slot_index: usize,
        slot: Slot,
    ) -> Result<(), InventoryError> {
        for player in self.get_current_players_in_container(server).await {
            let mut inventory = player.inventory().lock().await;
            let total_opened_containers = inventory.total_opened_containers;

            // Returns previous value
            inventory.state_id += 1;
            let packet = CSetContainerSlot::new(
                total_opened_containers as i8,
                (inventory.state_id) as i32,
                slot_index,
                &slot,
            );
            player.client.send_packet(&packet).await;
        }
        Ok(())
    }

    pub async fn send_whole_container_change(&self, server: &Server) -> Result<(), InventoryError> {
        let players = self.get_current_players_in_container(server).await;

        for player in players {
            let container = player.get_open_container(server).await;
            let mut container = match container.as_ref() {
                Some(container) => Some(container.lock().await),
                None => None,
            };
            player.set_container_content(container.as_deref_mut()).await;
        }
        Ok(())
    }

    pub async fn get_open_container(
        &self,
        server: &Server,
    ) -> Option<Arc<tokio::sync::Mutex<Box<dyn Container>>>> {
        match self.open_container.load() {
            Some(id) => server.try_get_container(self.entity_id(), id).await,
            None => None,
        }
    }

    async fn pickup_items(&self, item: &Item, mut amount: u32) {
        let max_stack = item.components.max_stack_size;
        let mut inventory = self.inventory().lock().await;
        let slots = inventory.slots_with_hotbar_first();

        let matching_slots = slots.filter_map(|slot| {
            if let Some(item_slot) = slot.as_mut() {
                (item_slot.item_id == item.id && item_slot.item_count < max_stack).then(|| {
                    let item_count = item_slot.item_count;
                    (item_slot, item_count)
                })
            } else {
                None
            }
        });

        for (slot, item_count) in matching_slots {
            if amount == 0 {
                return;
            }
            let amount_to_add = max_stack - item_count;
            if let Some(amount_left) = amount.checked_sub(u32::from(amount_to_add)) {
                amount = amount_left;
                *slot = ItemStack {
                    item_id: item.id,
                    item_count: item.components.max_stack_size,
                };
            } else {
                *slot = ItemStack {
                    item_id: item.id,
                    item_count: max_stack - (amount_to_add - amount as u8),
                };
                return;
            }
        }

        let empty_slots = inventory
            .slots_with_hotbar_first()
            .filter(|slot| slot.is_none());
        for slot in empty_slots {
            if amount == 0 {
                return;
            }
            if let Some(remaining_amount) = amount.checked_sub(u32::from(max_stack)) {
                amount = remaining_amount;
                *slot = Some(ItemStack {
                    item_id: item.id,
                    item_count: max_stack,
                });
            } else {
                *slot = Some(ItemStack {
                    item_id: item.id,
                    item_count: amount as u8,
                });
                return;
            }
        }
        log::warn!(
            "{amount} items were discarded because dropping them to the ground is not implemented"
        );
    }

    /// Add items to inventory if there's space, else drop them to the ground.
    ///
    /// This method automatically syncs changes with the client.
    pub async fn give_items(&self, item: &Item, amount: u32) {
        self.pickup_items(item, amount).await;
        self.set_container_content(None).await;
    }
}
