use num_traits::FromPrimitive;
use pumpkin_core::text::TextComponent;
use pumpkin_inventory::container_click;
use pumpkin_inventory::container_click::MouseClick;
use pumpkin_inventory::window_property::{WindowProperty, WindowPropertyTrait};
use pumpkin_inventory::{Container, WindowType};
use pumpkin_protocol::client::play::{
    CCloseContainer, COpenScreen, CSetContainerContent, CSetContainerProperty, CSetContainerSlot,
};
use pumpkin_protocol::server::play::SClickContainer;
use pumpkin_protocol::slot::Slot;
use pumpkin_world::item::ItemStack;

use crate::entity::player::Player;

impl Player {
    pub fn open_container(
        &mut self,
        window_type: WindowType,
        minecraft_menu_id: &str,
        window_title: Option<&str>,
        items: Option<Vec<Option<&ItemStack>>>,
        carried_item: Option<&ItemStack>,
    ) {
        let menu_protocol_id = (*pumpkin_world::global_registry::REGISTRY
            .get("minecraft:menu")
            .unwrap()
            .entries
            .get(minecraft_menu_id)
            .expect("Should be a valid menu id")
            .get("protocol_id")
            .unwrap())
        .into();
        let title = TextComponent::text(window_title.unwrap_or(window_type.default_title()));
        self.client.send_packet(&COpenScreen::new(
            (window_type.clone() as u8 + 1).into(),
            menu_protocol_id,
            title,
        ));
        self.set_container_content(window_type, items, carried_item);
    }

    pub fn set_container_content<'a>(
        &mut self,
        window_type: WindowType,
        items: Option<Vec<Option<&'a ItemStack>>>,
        carried_item: Option<&'a ItemStack>,
    ) {
        let slots: Vec<Slot> = {
            if let Some(mut items) = items {
                items.extend(self.inventory.slots());
                items
            } else {
                self.inventory.slots()
            }
            .into_iter()
            .map(|item| {
                if let Some(item) = item {
                    Slot::from(item)
                } else {
                    Slot::empty()
                }
            })
            .collect()
        };

        let carried_item = {
            if let Some(item) = carried_item {
                item.into()
            } else {
                Slot::empty()
            }
        };
        let packet =
            CSetContainerContent::new(window_type as u8 + 1, 0.into(), &slots, &carried_item);
        self.client.send_packet(&packet);
    }

    pub fn set_container_slot(
        &mut self,
        window_type: WindowType,
        slot: usize,
        item: Option<&ItemStack>,
    ) {
        self.client.send_packet(&CSetContainerSlot::new(
            window_type as i8,
            0,
            slot,
            &item.into(),
        ))
    }

    /// The official Minecraft client is weird, and will always just close *any* window that is opened when this gets sent
    pub fn close_container(&mut self, window_type: WindowType) {
        self.client
            .send_packet(&CCloseContainer::new(window_type as u8))
    }

    pub fn set_container_property<T: WindowPropertyTrait>(
        &mut self,
        window_type: WindowType,
        window_property: WindowProperty<T>,
    ) {
        let (id, value) = window_property.into_tuple();
        self.client
            .send_packet(&CSetContainerProperty::new(window_type as u8, id, value));
    }

    pub fn handle_click_container(&mut self, packet: SClickContainer) {
        use container_click::*;
        let click = Click {
            state_id: packet.state_id.0.try_into().unwrap(),
            changed_items: packet
                .array_of_changed_slots
                .into_iter()
                .map(|(slot, item)| {
                    let slot = slot.try_into().unwrap();
                    if let Some(item) = item.to_item() {
                        ItemChange::Add { slot, item }
                    } else {
                        ItemChange::Remove { slot }
                    }
                })
                .collect::<Vec<_>>(),
            window_type: WindowType::from_u8(packet.window_id).unwrap(),
            carried_item: packet.carried_item.to_item(),
            mode: ClickMode::new(
                packet
                    .mode
                    .0
                    .try_into()
                    .expect("Mode can only be between 0-6"),
                packet.button,
                packet.slot,
            ),
        };

        match click.mode.click_type {
            ClickType::MouseClick(mouse_click) => {
                self.mouse_click(mouse_click, click.window_type, click.mode.slot)
            }
            ClickType::ShiftClick => self.shift_mouse_click(click.window_type, click.mode.slot),
            _ => todo!(),
        }
        dbg!(&self.carried_item);
        let filled_inventory_slots = self
            .inventory
            .slots()
            .into_iter()
            .enumerate()
            .filter_map(|(slot, item)| item.map(|item| (slot, item.item_count)))
            .collect::<Vec<_>>();
        dbg!(filled_inventory_slots);
    }

    pub fn mouse_click(
        &mut self,
        mouse_click: MouseClick,
        window_type: WindowType,
        slot: container_click::Slot,
    ) {
        let Some(_) = &self.open_container else {
            // Inventory
            if window_type == WindowType::Generic9x1 {
                match slot {
                    container_click::Slot::Normal(slot) => {
                        self.inventory
                            .handle_item_change(&mut self.carried_item, slot, mouse_click)
                    }
                    container_click::Slot::OutsideInventory => (),
                };

                return;
            } else {
                return;
            }
        };
    }

    pub fn shift_mouse_click(&mut self, window_type: WindowType, slot: container_click::Slot) {
        let Some(_) = &self.open_container else {
            // Inventory
            if window_type == WindowType::Generic9x1 {
                match slot {
                    container_click::Slot::Normal(slot) => {
                        if let Some(item_in_pressed_slot) = self.inventory.slots()[slot] {
                            let mut slots = self.inventory.slots().into_iter().enumerate();
                            // Hotbar
                            let slots = if slot > 35 {
                                slots
                                    .skip(9)
                                    .find(|(_, slot)| {
                                        slot.is_none_or(|item| {
                                            item.item_id == item_in_pressed_slot.item_id
                                        })
                                    })
                                    .map(|(slot_num, _)| slot_num)
                            } else {
                                slots
                                    .skip(36)
                                    .rev()
                                    .find(|(_, slot)| {
                                        slot.is_none_or(|item| {
                                            item.item_id == item_in_pressed_slot.item_id
                                        })
                                    })
                                    .map(|(slot_num, _)| slot_num)
                            };
                            if let Some(slot) = slots {
                                let mut item_slot =
                                    self.inventory.slots()[slot].map(|i| i.to_owned());

                                self.inventory.handle_item_change(
                                    &mut item_slot,
                                    slot,
                                    MouseClick::Left,
                                );
                                *self.inventory.slots_mut()[slot] = item_slot;
                            }
                        }
                    }
                    container_click::Slot::OutsideInventory => (),
                };

                return;
            } else {
                return;
            }
        };
    }
}

/*impl<const SLOTS: usize> ContainerStruct<SLOTS> {
    pub fn opened_by_players(&mut self, server: Server) -> Vec<&Player> {

    }
}*/
