use pumpkin_core::text::TextComponent;
use pumpkin_inventory::window_property::{WindowProperty, WindowPropertyTrait};
use pumpkin_inventory::WindowType;
use pumpkin_protocol::client::play::{
    CCloseContainer, COpenScreen, CSetContainerContent, CSetContainerProperty, CSetContainerSlot,
};
use pumpkin_protocol::slot::Slot;
use pumpkin_world::item::Item;

use crate::entity::player::Player;

impl Player {
    pub fn open_container(
        &mut self,
        window_type: WindowType,
        minecraft_menu_id: &str,
        window_title: Option<&str>,
        items: Option<Vec<Option<&Item>>>,
        carried_item: Option<&Item>,
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
        items: Option<Vec<Option<&'a Item>>>,
        carried_item: Option<&'a Item>,
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
        item: Option<&Item>,
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
}
