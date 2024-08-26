use crate::container_click::MouseClick;
use crate::{handle_item_change, Container, WindowType};
use pumpkin_world::item::ItemStack;

#[allow(dead_code)]
#[derive(Debug)]
pub struct PlayerInventory {
    // Main Inventory + Hotbar
    crafting: [Option<ItemStack>; 4],
    crafting_output: Option<ItemStack>,
    items: [Option<ItemStack>; 36],
    armor: [Option<ItemStack>; 4],
    offhand: Option<ItemStack>,
    // current selected slot in hotbar
    selected: usize,
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
    pub fn set_slot(&mut self, slot: usize, item: Option<ItemStack>, item_allowed_override: bool) {
        match slot {
            0 => {
                // TODO: Add crafting check here
                self.crafting_output = item
            }
            1..=4 => self.crafting[slot - 1] = item,
            5..=8 => {
                match item {
                    None => self.armor[slot - 5] = None,
                    Some(item) => {
                        // TODO: Replace asserts with error handling
                        match slot - 5 {
                            0 => {
                                assert!(item.is_helmet() || item_allowed_override);
                                self.armor[0] = Some(item);
                            }
                            1 => {
                                assert!(item.is_chestplate() || item_allowed_override);
                                self.armor[1] = Some(item)
                            }
                            2 => {
                                assert!(item.is_leggings() || item_allowed_override);
                                self.armor[2] = Some(item);
                            }
                            3 => {
                                assert!(item.is_boots() || item_allowed_override);
                                self.armor[3] = Some(item)
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            9..=44 => {
                self.items[slot - 9] = item;
            }
            45 => {
                self.offhand = item;
            }
            _ => unreachable!(),
        }
    }
    pub fn get_slot(&mut self, slot: usize) -> &mut Option<ItemStack> {
        match slot {
            0 => {
                // TODO: Add crafting check here
                &mut self.crafting_output
            }
            1..=4 => &mut self.crafting[slot - 1],
            5..=8 => &mut self.armor[slot - 5],
            9..=44 => &mut self.items[slot - 9],
            45 => &mut self.offhand,
            _ => unreachable!(),
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
    fn window_type(&self) -> &WindowType {
        &WindowType::Generic9x1
    }

    fn handle_item_change(
        &mut self,
        carried_slot: &mut Option<ItemStack>,
        slot: usize,
        mouse_click: MouseClick,
    ) {
        let item_slot = self.get_slot(slot);
        handle_item_change(carried_slot, item_slot, mouse_click)
    }
}
