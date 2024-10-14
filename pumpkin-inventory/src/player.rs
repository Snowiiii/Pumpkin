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
    pub state_id: u32,
    // Notchian server wraps this value at 100, we can just keep it as a u8 that automatically wraps
    pub total_opened_containers: u8,
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
            state_id: 0,
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
        slot: usize,
        item: Option<ItemStack>,
        item_allowed_override: bool,
    ) -> Result<(), InventoryError> {
        if item_allowed_override {
            let Some(slot) = self.get_slot_mut(slot) else {
                return Err(InventoryError::InvalidSlot);
            };
            *slot = item;
        } else {
            let slot_condition = self.slot_condition(slot)?;
            if let Some(item) = item {
                if slot_condition(&item) {
                    let Some(slot) = self.get_slot_mut(slot) else {
                        return Err(InventoryError::InvalidSlot);
                    };
                    *slot = Some(item);
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

    pub fn set_selected(&mut self, slot: usize) {
        assert!((0..9).contains(&slot));
        self.selected = slot;
    }

    pub fn held_item(&self) -> Option<&ItemStack> {
        debug_assert!((0..9).contains(&self.selected));
        self.items[self.selected + 36 - 9].as_ref()
    }

    pub fn slots(&self) -> Box<dyn Iterator<Item = &Option<ItemStack>> + '_> {
        let crafting_output = [&self.crafting_output].into_iter();
        let crafting = self.crafting.iter();
        let armor = self.armor.iter();
        let items = self.items.iter();
        let offhand = [&self.offhand].into_iter();
        Box::new(crafting_output.chain(crafting.chain(armor.chain(items.chain(offhand)))))
    }

    pub fn slots_mut<'s>(&'s mut self) -> Box<dyn Iterator<Item = &mut Option<ItemStack>> + 's> {
        let crafting_output = [&mut self.crafting_output].into_iter();
        let crafting = self.crafting.iter_mut();
        let armor = self.armor.iter_mut();
        let items = self.items.iter_mut();
        let offhand = [&mut self.offhand].into_iter();
        Box::new(crafting_output.chain(crafting.chain(armor.chain(items.chain(offhand)))))
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
        let item_slot = self.get_slot_mut(slot).ok_or(InventoryError::InvalidSlot)?;
        if let Some(item) = carried_slot {
            if slot_condition(item) {
                handle_item_change(carried_slot, item_slot, mouse_click);
            }
        } else {
            handle_item_change(carried_slot, item_slot, mouse_click)
        }
        Ok(())
    }

    fn iter_slots_mut<'s>(
        &'s mut self,
    ) -> Box<(dyn Iterator<Item = &'s mut Option<ItemStack>> + 's)> {
        self.slots_mut()
    }

    fn iter_slots<'s>(&'s self) -> Box<(dyn Iterator<Item = &'s Option<ItemStack>> + 's)> {
        Box::new(self.slots())
    }

    fn iter_combinable_slots<'s>(&'s self) -> Box<dyn Iterator<Item = &'s Option<ItemStack>> + 's> {
        Box::new(self.items.iter())
    }

    fn iter_combinable_slots_mut<'s>(
        &'s mut self,
    ) -> Box<(dyn Iterator<Item = &'s mut Option<ItemStack>> + 's)> {
        Box::new(self.items.iter_mut())
    }

    fn get_slot(&self, slot: usize) -> Option<&Option<ItemStack>> {
        match slot {
            0 => {
                // TODO: Add crafting check here
                Some(&self.crafting_output)
            }
            1..=4 => Some(&self.crafting[slot - 1]),
            5..=8 => Some(&self.armor[slot - 5]),
            9..=44 => Some(&self.items[slot - 9]),
            45 => Some(&self.offhand),
            _ => None,
        }
    }

    fn get_slot_mut(&mut self, slot: usize) -> Option<&mut Option<ItemStack>> {
        match slot {
            0 => {
                // TODO: Add crafting check here
                Some(&mut self.crafting_output)
            }
            1..=4 => Some(&mut self.crafting[slot - 1]),
            5..=8 => Some(&mut self.armor[slot - 5]),
            9..=44 => Some(&mut self.items[slot - 9]),
            45 => Some(&mut self.offhand),
            _ => None,
        }
    }

    fn size(&self) -> usize {
        1 + // Crafting output
        self.crafting.len() +
        self.armor.len() +
        self.items.len() +
        1 // Offhand
    }
}
