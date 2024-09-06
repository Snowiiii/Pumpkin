use pumpkin_world::item::ItemStack;

pub struct Click {
    pub slot: Slot,
    pub click_type: ClickType,
}

impl Click {
    pub fn new(mode: u8, button: i8, slot: i16) -> Self {
        match mode {
            0 => Self::new_normal_click(button, slot),
            // Both buttons do the same here, so we omit it
            1 => Self::new_shift_click(slot),
            2 => Self::new_key_click(button, slot),
            3 => Self {
                click_type: ClickType::CreativePickItem,
                slot: Slot::Normal(slot.try_into().unwrap()),
            },
            4 => Self::new_drop_item(button),
            5 => Self::new_drag_item(button, slot),
            6 => Self {
                click_type: ClickType::DoubleClick,
                slot: Slot::Normal(slot.try_into().unwrap()),
            },
            // TODO: Error handling
            _ => unreachable!(),
        }
    }

    fn new_normal_click(button: i8, slot: i16) -> Self {
        let slot = match slot {
            -999 => Slot::OutsideInventory,
            _ => {
                // TODO: Error here
                let slot = slot.try_into();
                if let Ok(slot) = slot {
                    Slot::Normal(slot)
                } else {
                    Slot::OutsideInventory
                }
            }
        };
        let button = match button {
            0 => MouseClick::Left,
            1 => MouseClick::Right,
            // TODO: Error here
            _ => unreachable!(),
        };
        Self {
            click_type: ClickType::MouseClick(button),
            slot,
        }
    }

    fn new_shift_click(slot: i16) -> Self {
        Self {
            // TODO: Error handle this
            slot: Slot::Normal(slot.try_into().unwrap()),
            click_type: ClickType::ShiftClick,
        }
    }

    fn new_key_click(button: i8, slot: i16) -> Self {
        let key = match button {
            0..9 => KeyClick::Slot(button.try_into().unwrap()),
            40 => KeyClick::Offhand,
            // TODO: Error handling here
            _ => unreachable!(),
        };

        Self {
            click_type: ClickType::KeyClick(key),
            slot: Slot::Normal(slot.try_into().unwrap()),
        }
    }

    fn new_drop_item(button: i8) -> Self {
        let drop_type = match button {
            0 => DropType::SingleItem,
            1 => DropType::FullStack,
            // TODO: Error handling
            _ => unreachable!(),
        };
        Self {
            click_type: ClickType::DropType(drop_type),
            slot: Slot::OutsideInventory,
        }
    }

    fn new_drag_item(button: i8, slot: i16) -> Self {
        let state = match button {
            0 => MouseDragState::Start(MouseDragType::Left),
            4 => MouseDragState::Start(MouseDragType::Right),
            8 => MouseDragState::Start(MouseDragType::Middle),
            1 | 5 | 9 => MouseDragState::AddSlot(slot.try_into().unwrap()),
            2 | 6 | 10 => MouseDragState::End,
            // TODO: Error handling
            _ => unreachable!(),
        };
        Self {
            slot: match &state {
                MouseDragState::AddSlot(slot) => Slot::Normal(*slot),
                _ => Slot::OutsideInventory,
            },
            click_type: ClickType::MouseDrag { drag_state: state },
        }
    }
}

pub enum ClickType {
    MouseClick(MouseClick),
    ShiftClick,
    KeyClick(KeyClick),
    CreativePickItem,
    DropType(DropType),
    MouseDrag { drag_state: MouseDragState },
    DoubleClick,
}
#[derive(Debug, PartialEq, Eq)]
pub enum MouseClick {
    Left,
    Right,
}

pub enum KeyClick {
    Slot(u8),
    Offhand,
}
#[derive(Copy, Clone)]
pub enum Slot {
    Normal(usize),
    OutsideInventory,
}

pub enum DropType {
    SingleItem,
    FullStack,
}
#[derive(Debug, PartialEq)]
pub enum MouseDragType {
    Left,
    Right,
    Middle,
}
#[derive(PartialEq)]
pub enum MouseDragState {
    Start(MouseDragType),
    AddSlot(usize),
    End,
}

pub enum ItemChange {
    Remove { slot: usize },
    Add { slot: usize, item: ItemStack },
}
