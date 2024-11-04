use std::sync::atomic::AtomicU32;

use crate::container_click::MouseClick;
use crate::{handle_item_change, Container, InventoryError, WindowType};
use pumpkin_world::item::ItemStack;

pub struct PlayerInventory {
    // Main Inventory + Hotbar
    crafting: [Option<ItemStack>; 4],
    crafting_output: Option<ItemStack>,
    items: [Option<ItemStack>; 36],
    armor: [Option<ItemStack>; 4],
    offhand: Option<ItemStack>,
    // current selected slot in hotbar
    selected: usize,
    pub state_id: AtomicU32,
    // Notchian server wraps this value at 100, we can just keep it as a u8 that automatically wraps
    pub total_opened_containers: i32,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            crafting: [None; 4],
            crafting_output: None,
            items: [None; 36],
            armor: [None; 4],
            offhand: None,
            // TODO: What when player spawns in with an different index ?
            selected: 0,
            state_id: AtomicU32::new(0),
            total_opened_containers: 2,
        }
    }
    /// Set the contents of an item in a slot
    ///
    /// ## Slot
    /// The slot according to https://wiki.vg/Inventory#Player_Inventory
    ///
    /// ## Item
    /// The optional item to place in the slot
    ///
    /// ## Item allowed override
    /// An override, which when enabled, makes it so that invalid items, can be placed in slots they normally can't.
    /// Useful functionality for plugins in the future.
    pub fn set_slot(
        &mut self,
        slot: u16,
        item: Option<ItemStack>,
        item_allowed_override: bool,
    ) -> Result<(), InventoryError> {
        if !(0..=45).contains(&slot) {
            return Err(InventoryError::InvalidSlot);
        }

        match item_allowed_override {
            true => {
                *self.all_slots()[slot as usize] = item;
            }
            false => {
                let slot = slot as usize;
                let slot_condition = self.slot_condition(slot)?;
                if let Some(item) = item {
                    if slot_condition(&item) {
                        self.all_slots()[slot] = &mut Some(item);
                    }
                }
            }
        }

        Ok(())
    }
    #[allow(clippy::type_complexity)]
    pub fn slot_condition(
        &self,
        slot: usize,
    ) -> Result<Box<dyn Fn(&ItemStack) -> bool>, InventoryError> {
        if !(0..=45).contains(&slot) {
            return Err(InventoryError::InvalidSlot);
        }

        Ok(Box::new(match slot {
            0..=4 | 9..=45 => |_| true,
            5 => |item: &ItemStack| item.is_helmet(),
            6 => |item: &ItemStack| item.is_chestplate(),
            7 => |item: &ItemStack| item.is_leggings(),
            8 => |item: &ItemStack| item.is_boots(),
            _ => unreachable!(),
        }))
    }
    pub fn get_slot(&mut self, slot: usize) -> Result<&mut Option<ItemStack>, InventoryError> {
        match slot {
            0 => {
                // TODO: Add crafting check here
                Ok(&mut self.crafting_output)
            }
            1..=4 => Ok(&mut self.crafting[slot - 1]),
            5..=8 => Ok(&mut self.armor[slot - 5]),
            9..=44 => Ok(&mut self.items[slot - 9]),
            45 => Ok(&mut self.offhand),
            _ => Err(InventoryError::InvalidSlot),
        }
    }
    pub fn set_selected(&mut self, slot: usize) {
        assert!((0..9).contains(&slot));
        self.selected = slot;
    }

    pub fn held_item(&self) -> Option<&ItemStack> {
        debug_assert!((0..9).contains(&self.selected));
        self.items[self.selected + 36 - 9].as_ref()
    }

    pub fn slots(&self) -> Vec<Option<&ItemStack>> {
        let mut slots = vec![self.crafting_output.as_ref()];
        slots.extend(self.crafting.iter().map(|c| c.as_ref()));
        slots.extend(self.armor.iter().map(|c| c.as_ref()));
        slots.extend(self.items.iter().map(|c| c.as_ref()));
        slots.push(self.offhand.as_ref());
        slots
    }

    pub fn slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        let mut slots = vec![&mut self.crafting_output];
        slots.extend(self.crafting.iter_mut());
        slots.extend(self.armor.iter_mut());
        slots.extend(self.items.iter_mut());
        slots.push(&mut self.offhand);
        slots
    }
}

impl Container for PlayerInventory {
    fn window_type(&self) -> &'static WindowType {
        &WindowType::Generic9x1
    }

    fn window_name(&self) -> &'static str {
        // We never send an OpenContainer with inventory, so it has no name.
        ""
    }

    fn handle_item_change(
        &mut self,
        carried_slot: &mut Option<ItemStack>,
        slot: usize,
        mouse_click: MouseClick,
    ) -> Result<(), InventoryError> {
        let slot_condition = self.slot_condition(slot)?;
        let item_slot = self.get_slot(slot)?;
        if let Some(item) = carried_slot {
            if slot_condition(item) {
                handle_item_change(carried_slot, item_slot, mouse_click);
            }
        } else {
            handle_item_change(carried_slot, item_slot, mouse_click)
        }
        Ok(())
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.slots_mut()
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        self.slots()
    }

    fn all_combinable_slots(&self) -> Vec<Option<&ItemStack>> {
        self.items.iter().map(|item| item.as_ref()).collect()
    }

    fn all_combinable_slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.items.iter_mut().collect()
    }
}
