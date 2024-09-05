use crate::container_click::MouseClick;
use crate::player::PlayerInventory;
use num_derive::{FromPrimitive, ToPrimitive};
use pumpkin_world::item::ItemStack;

pub mod container_click;
pub mod drag_handler;
mod error;
mod open_container;
pub mod player;
pub mod window_property;

pub use error::InventoryError;
pub use open_container::OpenContainer;

/// https://wiki.vg/Inventory
#[derive(Debug, ToPrimitive, FromPrimitive, Clone, Copy, Eq, PartialEq)]
pub enum WindowType {
    // not used
    Generic9x1,
    // not used
    Generic9x2,
    // General-purpose 3-row inventory. Used by Chest, minecart with chest, ender chest, and barrel
    Generic9x3,
    // not used
    Generic9x4,
    // not used
    Generic9x5,
    // Used by large chests
    Generic9x6,
    // General-purpose 3-by-3 square inventory, used by Dispenser and Dropper
    Generic3x3,
    // General-purpose 3-by-3 square inventory, used by the Crafter
    Craft3x3,
    Anvil,
    Beacon,
    BlastFurnace,
    BrewingStand,
    CraftingTable,
    EnchantmentTable,
    Furnace,
    Grindstone,
    // Hopper or minecart with hopper
    Hopper,
    Lectern,
    Loom,
    // Villager, Wandering Trader
    Merchant,
    ShulkerBox,
    SmithingTable,
    Smoker,
    CartographyTable,
    Stonecutter,
}

impl WindowType {
    pub const fn default_title(&self) -> &'static str {
        // TODO: Add titles here:
        /*match self {
            _ => "WINDOW TITLE",
        }*/
        "WINDOW TITLE"
    }
}
// Container needs Sync + Send to be able to be in async Server
pub trait Container: Sync + Send {
    fn window_type(&self) -> &'static WindowType;

    fn handle_item_change(
        &mut self,
        carried_item: &mut Option<ItemStack>,
        slot: usize,
        mouse_click: MouseClick,
    ) -> Result<(), InventoryError> {
        let mut all_slots = self.all_slots();
        if slot < all_slots.len() {
            Err(InventoryError::InvalidSlot)?
        }
        handle_item_change(carried_item, all_slots[slot], mouse_click);
        Ok(())
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>>;

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>>;

    fn all_combinable_slots(&self) -> Vec<Option<&ItemStack>> {
        self.all_slots_ref()
    }

    fn all_combinable_slots_mut(&mut self) -> Vec<&mut Option<ItemStack>> {
        self.all_slots()
    }

    fn advance_state_id(&mut self) -> i32;

    fn reset_state_id(&mut self);
    fn state_id(&self) -> i32;

    fn print_all_contents(&self) {
        self.all_slots_ref()
            .into_iter()
            .enumerate()
            .filter_map(|(slot, item)| item.map(|item| (slot, item)))
            .for_each(|(slot, item)| {
                dbg!(slot, item);
            });
    }

    fn internal_pumpkin_id(&self) -> u64 {
        0
    }
}

pub fn handle_item_take(
    carried_item: &mut Option<ItemStack>,
    item_slot: &mut Option<ItemStack>,
    mouse_click: MouseClick,
) {
    let Some(item) = item_slot else {
        return;
    };
    let mut new_item = *item;

    match mouse_click {
        MouseClick::Left => {
            *item_slot = None;
        }
        MouseClick::Right => {
            let half = item.item_count / 2;
            item.item_count -= half;
            new_item.item_count = half;
        }
    }
    *carried_item = Some(new_item);
}
pub fn handle_item_change(
    carried_slot: &mut Option<ItemStack>,
    current_slot: &mut Option<ItemStack>,
    mouse_click: MouseClick,
) {
    match (current_slot.as_mut(), carried_slot.as_mut()) {
        // Swap or combine current and carried
        (Some(current), Some(carried)) => {
            if current.item_id == carried.item_id {
                combine_stacks(carried_slot, current, mouse_click);
            } else if mouse_click == MouseClick::Left {
                let carried = *carried;
                *carried_slot = Some(current.to_owned());
                *current_slot = Some(carried.to_owned());
            }
        }
        // Put held stack into empty slot
        (None, Some(carried)) => match mouse_click {
            MouseClick::Left => {
                *current_slot = Some(carried.to_owned());
                *carried_slot = None;
            }
            MouseClick::Right => {
                carried.item_count -= 1;
                let mut new = *carried;
                new.item_count = 1;
                *current_slot = Some(new);
            }
        },
        // Take stack into carried
        (Some(_current), None) => handle_item_take(carried_slot, current_slot, mouse_click),
        (None, None) => (),
    }
}

pub fn combine_stacks(
    carried_slot: &mut Option<ItemStack>,
    slot: &mut ItemStack,
    mouse_click: MouseClick,
) {
    let Some(carried_item) = carried_slot else {
        return;
    };

    let carried_change = match mouse_click {
        MouseClick::Left => carried_item.item_count,
        MouseClick::Right => 1,
    };

    // TODO: Check for item stack max size here
    if slot.item_count + carried_change <= 64 {
        slot.item_count += carried_change;
        carried_item.item_count -= carried_change;
        if carried_item.item_count == 0 {
            *carried_slot = None;
        }
    } else {
        let left_over = slot.item_count + carried_change - 64;
        slot.item_count = 64;
        carried_item.item_count = left_over;
    }
}

pub struct OptionallyCombinedContainer<'a, 'b> {
    container: Option<&'a mut Box<dyn Container>>,
    inventory: &'b mut PlayerInventory,
}
impl<'a, 'b> OptionallyCombinedContainer<'a, 'b> {
    pub fn new(
        player_inventory: &'b mut PlayerInventory,
        container: Option<&'a mut Box<dyn Container>>,
    ) -> Self {
        Self {
            inventory: player_inventory,
            container,
        }
    }
}

impl<'a> Container for OptionallyCombinedContainer<'a, 'a> {
    fn window_type(&self) -> &'static WindowType {
        if let Some(container) = &self.container {
            container.window_type()
        } else {
            &WindowType::Generic9x1
        }
    }

    fn all_slots(&mut self) -> Vec<&mut Option<ItemStack>> {
        let slots = match &mut self.container {
            Some(container) => {
                let mut slots = container.all_slots();
                slots.extend(self.inventory.all_combinable_slots_mut());
                slots
            }
            None => self.inventory.all_slots(),
        };
        dbg!(slots.len());
        slots
    }

    fn all_slots_ref(&self) -> Vec<Option<&ItemStack>> {
        match &self.container {
            Some(container) => {
                let mut slots = container.all_slots_ref();
                slots.extend(self.inventory.all_combinable_slots());
                slots
            }
            None => self.inventory.all_slots_ref(),
        }
    }

    fn advance_state_id(&mut self) -> i32 {
        self.inventory.advance_state_id()
    }

    fn reset_state_id(&mut self) {
        self.inventory.reset_state_id()
    }

    fn state_id(&self) -> i32 {
        self.inventory.state_id()
    }
}
