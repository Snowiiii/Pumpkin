use crate::entity::player::Player;
use crate::server::Server;
use itertools::Itertools;
use parking_lot::Mutex;
use pumpkin_core::text::TextComponent;
use pumpkin_core::GameMode;
use pumpkin_inventory::container_click::{
    Click, ClickType, KeyClick, MouseClick, MouseDragState, MouseDragType,
};
use pumpkin_inventory::drag_handler::DragHandler;
use pumpkin_inventory::window_property::{WindowProperty, WindowPropertyTrait};
use pumpkin_inventory::Container;
use pumpkin_inventory::{container_click, InventoryError, OptionallyCombinedContainer};
use pumpkin_protocol::client::play::{
    CCloseContainer, COpenScreen, CSetContainerContent, CSetContainerProperty, CSetContainerSlot,
};
use pumpkin_protocol::server::play::SClickContainer;
use pumpkin_protocol::slot::Slot;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

impl Player {
    pub fn open_container(&self, server: &Server, minecraft_menu_id: &str) {
        let mut inventory = self.inventory.lock();
        inventory.state_id = 0;
        let total_opened_containers = inventory.total_opened_containers;
        let container = self.get_open_container(server);
        let mut container = container.as_ref().map(|container| container.lock());
        let menu_protocol_id = (*pumpkin_world::global_registry::REGISTRY
            .get("minecraft:menu")
            .unwrap()
            .entries
            .get(minecraft_menu_id)
            .expect("Should be a valid menu id")
            .get("protocol_id")
            .unwrap())
        .into();
        let window_title = container
            .as_ref()
            .map(|container| container.window_name())
            .unwrap_or_else(|| inventory.window_name());
        let title = TextComponent::text(window_title);

        self.client.send_packet(&COpenScreen::new(
            total_opened_containers.into(),
            menu_protocol_id,
            title,
        ));
        drop(inventory);
        self.set_container_content(container.as_deref_mut());
    }

    pub fn set_container_content(&self, container: Option<&mut Box<dyn Container>>) {
        let mut inventory = self.inventory.lock();

        let total_opened_containers = inventory.total_opened_containers;
        let container = OptionallyCombinedContainer::new(&mut inventory, container);

        let slots = container.iter_slots().map(Slot::from).collect_vec();

        let carried_item = self
            .carried_item
            .lock()
            .map_or_else(Slot::empty, Slot::from);

        // Gets the previous value
        inventory.state_id += 1;
        let packet = CSetContainerContent::new(
            total_opened_containers,
            (inventory.state_id as i32).into(),
            &slots,
            &carried_item,
        );
        self.client.send_packet(&packet);
    }

    /// The official Minecraft client is weird, and will always just close *any* window that is opened when this gets sent
    pub fn close_container(&self) {
        let mut inventory = self.inventory.lock();
        inventory.total_opened_containers += 1;
        self.client
            .send_packet(&CCloseContainer::new(inventory.total_opened_containers))
    }

    pub fn set_container_property<T: WindowPropertyTrait>(
        &mut self,
        window_property: WindowProperty<T>,
    ) {
        let (id, value) = window_property.into_tuple();
        self.client.send_packet(&CSetContainerProperty::new(
            self.inventory.lock().total_opened_containers,
            id,
            value,
        ));
    }

    pub async fn handle_click_container(
        &self,
        server: &Arc<Server>,
        packet: SClickContainer,
    ) -> Result<(), InventoryError> {
        let opened_container = self.get_open_container(server);
        let mut opened_container = opened_container.as_ref().map(|container| container.lock());
        let drag_handler = &server.drag_handler;

        let state_id = self.inventory.lock().state_id;
        // This is just checking for regular desync, client hasn't done anything malicious
        if state_id != packet.state_id.0 as u32 {
            self.set_container_content(opened_container.as_deref_mut());
            return Ok(());
        }

        if opened_container.is_some() {
            if packet.window_id != self.inventory.lock().total_opened_containers {
                return Err(InventoryError::ClosedContainerInteract(self.entity_id()));
            }
        } else if packet.window_id != 0 {
            return Err(InventoryError::ClosedContainerInteract(self.entity_id()));
        }

        let click = Click::new(
            packet
                .mode
                .0
                .try_into()
                .expect("Mode can only be between 0-6"),
            packet.button,
            packet.slot,
        )?;
        let mut update_whole_container = false;

        match click.click_type {
            ClickType::MouseClick(mouse_click) => {
                self.mouse_click(opened_container.as_deref_mut(), mouse_click, click.slot)
            }
            ClickType::ShiftClick => {
                self.shift_mouse_click(opened_container.as_deref_mut(), click.slot)
            }
            ClickType::KeyClick(key_click) => match click.slot {
                container_click::Slot::Normal(slot) => {
                    self.number_button_pressed(opened_container.as_deref_mut(), key_click, slot)
                }
                container_click::Slot::OutsideInventory => Err(InventoryError::InvalidPacket),
            },
            ClickType::CreativePickItem => {
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.creative_pick_item(opened_container.as_deref_mut(), slot)
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::DoubleClick => {
                update_whole_container = true;
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.double_click(opened_container.as_deref_mut(), slot)
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::MouseDrag { drag_state } => {
                if drag_state == MouseDragState::End {
                    update_whole_container = true;
                }
                self.mouse_drag(drag_handler, opened_container.as_deref_mut(), drag_state)
            }
            ClickType::DropType(_drop_type) => {
                dbg!("todo");
                Ok(())
            }
        }?;
        if let Some(mut opened_container) = opened_container {
            if update_whole_container {
                drop(opened_container);
                self.send_whole_container_change(server).await?;
            } else if let container_click::Slot::Normal(slot_index) = click.slot {
                let mut inventory = self.inventory.lock();
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

    fn mouse_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_click: MouseClick,
        slot: container_click::Slot,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory.lock();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        match slot {
            container_click::Slot::Normal(slot) => {
                let mut carried_item = self.carried_item.lock();
                container.handle_item_change(&mut carried_item, slot, mouse_click)?;
            }
            container_click::Slot::OutsideInventory => (),
        }
        Ok(())
    }

    fn shift_mouse_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: container_click::Slot,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory.lock();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        match slot {
            // TODO: this is definitely not working as intended
            container_click::Slot::Normal(slot) => {
                let Some(Some(item_in_pressed_slot)) = container.get_slot(slot) else {
                    return Ok(());
                };
                let item_in_pressed_slot = *item_in_pressed_slot;
                // Hotbar
                let find_condition = |(slot_number, slot): (usize, &mut Option<ItemStack>)| {
                    // TODO: Check for max item count here
                    match slot {
                        Some(item) => {
                            if item.item_id == item_in_pressed_slot.item_id && item.item_count != 64
                            {
                                Some(slot_number)
                            } else {
                                None
                            }
                        }
                        None => Some(slot_number),
                    }
                };
                let all_slots = container.iter_slots_mut().enumerate();
                let slots = if slot > 35 {
                    all_slots.skip(9).find_map(find_condition)
                } else {
                    all_slots.skip(36).filter_map(find_condition).last()
                };
                if let Some(slot) = slots {
                    container.handle_item_change(
                        &mut self.carried_item.lock(),
                        slot,
                        MouseClick::Left,
                    )?;
                }
            }
            container_click::Slot::OutsideInventory => (),
        };
        Ok(())
    }

    fn number_button_pressed(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        key_click: KeyClick,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let changing_slot = match key_click {
            KeyClick::Slot(slot) => slot,
            KeyClick::Offhand => 45,
        };
        let mut inventory = self.inventory.lock();
        let mut changing_item_slot = inventory
            .get_slot_mut(changing_slot as usize)
            .ok_or(InventoryError::InvalidSlot)?
            .to_owned();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        container.handle_item_change(&mut changing_item_slot, slot, MouseClick::Left)?;
        *inventory
            .get_slot_mut(changing_slot as usize)
            .ok_or(InventoryError::InvalidSlot)? = changing_item_slot;
        Ok(())
    }

    fn creative_pick_item(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: usize,
    ) -> Result<(), InventoryError> {
        if self.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let mut inventory = self.inventory.lock();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);
        if let Some(Some(item)) = container.get_slot_mut(slot) {
            *self.carried_item.lock() = Some(*item);
        }
        Ok(())
    }

    fn double_click(
        &self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let mut inventory = self.inventory.lock();
        let mut container = OptionallyCombinedContainer::new(&mut inventory, opened_container);

        let Some(item) = container.get_slot_mut(slot) else {
            return Ok(());
        };
        let Some(mut carried_item) = *item else {
            return Ok(());
        };
        *item = None;
        let slots = container.iter_slots_mut();
        for slot in slots.filter_map(|slot| slot.as_mut()) {
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
        *self.carried_item.lock() = Some(carried_item);
        Ok(())
    }

    fn mouse_drag(
        &self,
        drag_handler: &DragHandler,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_drag_state: MouseDragState,
    ) -> Result<(), InventoryError> {
        let player_id = self.entity_id();
        let container_id = opened_container
            .as_ref()
            .map(|container| container.internal_pumpkin_id())
            .unwrap_or(player_id as u64);
        match mouse_drag_state {
            MouseDragState::Start(drag_type) => {
                if drag_type == MouseDragType::Middle && self.gamemode.load() != GameMode::Creative
                {
                    Err(InventoryError::PermissionError)?
                }
                drag_handler.new_drag(container_id, player_id, drag_type)
            }
            MouseDragState::AddSlot(slot) => drag_handler.add_slot(container_id, player_id, slot),
            MouseDragState::End => {
                let mut inventory = self.inventory.lock();
                let mut container =
                    OptionallyCombinedContainer::new(&mut inventory, opened_container);
                let res = drag_handler.apply_drag(
                    &mut self.carried_item.lock(),
                    &mut container,
                    &container_id,
                    player_id,
                );
                res
            }
        }
    }

    async fn get_current_players_in_container(&self, server: &Server) -> Vec<Arc<Self>> {
        let player_ids = {
            let open_containers = server.open_containers.read();
            open_containers
                .get(&self.open_container.load().unwrap())
                .unwrap()
                .all_player_ids()
                .into_iter()
                .filter(|player_id| *player_id != self.entity_id())
                .collect_vec()
        };
        let player_token = self.client.token;

        // TODO: Figure out better way to get only the players from player_ids
        // Also refactor out a better method to get individual advanced state ids

        let players = self
            .living_entity
            .entity
            .world
            .current_players
            .lock()
            .iter()
            .filter_map(|(token, player)| {
                if *token != player_token {
                    let entity_id = player.entity_id();
                    if player_ids.contains(&entity_id) {
                        Some(player.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect_vec();
        players
    }

    async fn send_container_changes(
        &self,
        server: &Server,
        slot_index: usize,
        slot: Slot,
    ) -> Result<(), InventoryError> {
        for player in self.get_current_players_in_container(server).await {
            let mut inventory = player.inventory.lock();
            let total_opened_containers = inventory.total_opened_containers;

            // Returns previous value
            inventory.state_id += 1;
            let packet = CSetContainerSlot::new(
                total_opened_containers as i8,
                inventory.state_id as i32,
                slot_index,
                &slot,
            );
            player.client.send_packet(&packet);
        }
        Ok(())
    }

    async fn send_whole_container_change(&self, server: &Server) -> Result<(), InventoryError> {
        let players = self.get_current_players_in_container(server).await;

        for player in players {
            let container = player.get_open_container(server);
            let mut container = container.as_ref().map(|v| v.lock());
            player.set_container_content(container.as_deref_mut());
        }
        Ok(())
    }

    pub fn get_open_container(&self, server: &Server) -> Option<Arc<Mutex<Box<dyn Container>>>> {
        self.open_container
            .load()
            .map_or_else(|| None, |id| server.try_get_container(self.entity_id(), id))
    }
}
