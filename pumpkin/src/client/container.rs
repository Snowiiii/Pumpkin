use crate::entity::player::Player;
use crate::server::Server;
use itertools::Itertools;
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
use pumpkin_world::item::ItemStack;
use std::sync::Mutex;

impl Player {
    pub fn open_container(
        &mut self,
        server: &mut Server,
        minecraft_menu_id: &str,
        window_title: Option<&str>,
    ) {
        self.inventory.state_id = 0;
        let total_opened_containers = self.inventory.total_opened_containers;
        let mut container = self
            .get_open_container(server)
            .map(|container| container.lock().unwrap());
        let menu_protocol_id = (*pumpkin_world::global_registry::REGISTRY
            .get("minecraft:menu")
            .unwrap()
            .entries
            .get(minecraft_menu_id)
            .expect("Should be a valid menu id")
            .get("protocol_id")
            .unwrap())
        .into();
        let window_type = match &container {
            Some(container) => container.window_type(),
            None => &WindowType::Generic9x1,
        }
        .to_owned();
        let title = TextComponent::text(window_title.unwrap_or(window_type.default_title()));

        self.client.send_packet(&COpenScreen::new(
            total_opened_containers.into(),
            menu_protocol_id,
            title,
        ));
        self.set_container_content(container.as_deref_mut());
    }

    pub fn set_container_content(&mut self, container: Option<&mut Box<dyn Container>>) {
        let total_opened_containers = self.inventory.total_opened_containers;
        let container = OptionallyCombinedContainer::new(&mut self.inventory, container);

        let slots = container
            .all_slots_ref()
            .into_iter()
            .map(Slot::from)
            .collect_vec();

        let carried_item = {
            if let Some(item) = self.carried_item.as_ref() {
                item.into()
            } else {
                Slot::empty()
            }
        };
        self.inventory.state_id += 1;
        let packet = CSetContainerContent::new(
            total_opened_containers,
            (self.inventory.state_id as i32).into(),
            &slots,
            &carried_item,
        );
        self.client.send_packet(&packet);
    }

    /// The official Minecraft client is weird, and will always just close *any* window that is opened when this gets sent
    pub fn close_container(&mut self) {
        self.inventory.total_opened_containers += 1;
        self.client.send_packet(&CCloseContainer::new(
            self.inventory.total_opened_containers,
        ))
    }

    pub fn set_container_property<T: WindowPropertyTrait>(
        &mut self,
        window_property: WindowProperty<T>,
    ) {
        let (id, value) = window_property.into_tuple();
        self.client.send_packet(&CSetContainerProperty::new(
            self.inventory.total_opened_containers,
            id,
            value,
        ));
    }

    pub async fn handle_click_container(
        &mut self,
        server: &mut Server,
        packet: SClickContainer,
    ) -> Result<(), InventoryError> {
        let mut opened_container = self
            .get_open_container(server)
            .map(|container| container.lock().unwrap());
        let drag_handler = &server.drag_handler;

        let state_id = self.inventory.state_id;
        // This is just checking for regular desync, client hasn't done anything malicious
        if state_id != packet.state_id.0 as u32 {
            self.set_container_content(opened_container.as_deref_mut());
            return Ok(());
        }

        if opened_container.is_some() {
            if packet.window_id != self.inventory.total_opened_containers {
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
        );

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
                if let container_click::Slot::Normal(slot) = click.slot {
                    self.double_click(slot)
                } else {
                    Err(InventoryError::InvalidPacket)
                }
            }
            ClickType::MouseDrag { drag_state } => {
                self.mouse_drag(drag_handler, opened_container.as_deref_mut(), drag_state)
            }
            ClickType::DropType(_drop_type) => todo!(),
        }?;
        if let Some(mut opened_container) = opened_container {
            if let container_click::Slot::Normal(slot_index) = click.slot {
                let combined_container = OptionallyCombinedContainer::new(
                    &mut self.inventory,
                    Some(&mut opened_container),
                );
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
        &mut self,
        opened_container: Option<&mut Box<dyn Container>>,
        mouse_click: MouseClick,
        slot: container_click::Slot,
    ) -> Result<(), InventoryError> {
        let mut container = OptionallyCombinedContainer::new(&mut self.inventory, opened_container);

        match slot {
            container_click::Slot::Normal(slot) => {
                container.handle_item_change(&mut self.carried_item, slot, mouse_click)
            }
            container_click::Slot::OutsideInventory => Ok(()),
        }
    }

    fn shift_mouse_click(
        &mut self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: container_click::Slot,
    ) -> Result<(), InventoryError> {
        let mut container = OptionallyCombinedContainer::new(&mut self.inventory, opened_container);

        match slot {
            container_click::Slot::Normal(slot) => {
                let all_slots = container.all_slots();
                if let Some(item_in_pressed_slot) = all_slots[slot].to_owned() {
                    let slots = all_slots.into_iter().enumerate();
                    // Hotbar
                    let find_condition = |(slot_number, slot): (usize, &mut Option<ItemStack>)| {
                        // TODO: Check for max item count here
                        match slot {
                            Some(item) => {
                                if item.item_id == item_in_pressed_slot.item_id
                                    && item.item_count != 64
                                {
                                    Some(slot_number)
                                } else {
                                    None
                                }
                            }
                            None => Some(slot_number),
                        }
                    };

                    let slots = if slot > 35 {
                        slots.skip(9).find_map(find_condition)
                    } else {
                        slots.skip(36).rev().find_map(find_condition)
                    };
                    if let Some(slot) = slots {
                        let mut item_slot = container.all_slots()[slot].map(|i| i.to_owned());
                        container.handle_item_change(&mut item_slot, slot, MouseClick::Left)?;
                        *container.all_slots()[slot] = item_slot;
                    }
                }
            }
            container_click::Slot::OutsideInventory => (),
        };
        Ok(())
    }

    fn number_button_pressed(
        &mut self,
        opened_container: Option<&mut Box<dyn Container>>,
        key_click: KeyClick,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let changing_slot = match key_click {
            KeyClick::Slot(slot) => slot,
            KeyClick::Offhand => 45,
        };
        let mut changing_item_slot = self.inventory.get_slot(changing_slot as usize)?.to_owned();
        let mut container = OptionallyCombinedContainer::new(&mut self.inventory, opened_container);

        container.handle_item_change(&mut changing_item_slot, slot, MouseClick::Left)?;
        *self.inventory.get_slot(changing_slot as usize)? = changing_item_slot;
        Ok(())
    }

    fn creative_pick_item(
        &mut self,
        opened_container: Option<&mut Box<dyn Container>>,
        slot: usize,
    ) -> Result<(), InventoryError> {
        if self.gamemode != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let mut container = OptionallyCombinedContainer::new(&mut self.inventory, opened_container);
        if let Some(Some(item)) = container.all_slots().get_mut(slot) {
            self.carried_item = Some(item.to_owned())
        }
        Ok(())
    }

    fn double_click(&mut self, _slot: usize) -> Result<(), InventoryError> {
        Ok(())
    }

    fn mouse_drag(
        &mut self,
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
                if drag_type == MouseDragType::Middle && self.gamemode != GameMode::Creative {
                    Err(InventoryError::PermissionError)?
                }
                drag_handler.new_drag(container_id, player_id, drag_type)
            }
            MouseDragState::AddSlot(slot) => drag_handler.add_slot(container_id, player_id, slot),
            MouseDragState::End => {
                let mut container =
                    OptionallyCombinedContainer::new(&mut self.inventory, opened_container);
                drag_handler.apply_drag(
                    &mut self.carried_item,
                    &mut container,
                    &container_id,
                    player_id,
                )
            }
        }
    }

    async fn send_container_changes(
        &mut self,
        server: &Server,
        slot_index: usize,
        slot: Slot,
    ) -> Result<(), InventoryError> {
        let player_ids = server
            .open_containers
            .get(&self.open_container.unwrap())
            .unwrap()
            .all_player_ids()
            .into_iter()
            .filter(|player_id| *player_id != self.entity_id())
            .collect_vec();
        let player_token = self.client.token;

        // TODO: Figure out better way to get only the players from player_ids
        // Also refactor out a better method to get individual advanced state ids

        let world = self.world.lock().await;
        let players = world
            .current_players
            .iter()
            .filter_map(|(token, player)| {
                if *token != player_token {
                    let entity_id = player.lock().unwrap().entity_id();
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
        drop(world);
        for player in players {
            let mut player = player.lock().unwrap();
            let total_opened_containers = player.inventory.total_opened_containers;

            player.inventory.state_id += 1;
            let packet = CSetContainerSlot::new(
                total_opened_containers as i8,
                player.inventory.state_id as i32,
                slot_index,
                &slot,
            );
            player.client.send_packet(&packet);
        }

        Ok(())
    }

    pub fn get_open_container<'a>(
        &self,
        server: &'a Server,
    ) -> Option<&'a Mutex<Box<dyn Container>>> {
        if let Some(id) = self.open_container {
            server.try_get_container(self.entity_id(), id)
        } else {
            None
        }
    }
}
