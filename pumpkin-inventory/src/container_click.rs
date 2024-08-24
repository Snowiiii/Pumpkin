pub struct ClickMode {
    slot: Slot,
    click_type: ClickType,
}

impl ClickMode {
    fn new(mode: u8, button: i8, slot: i16, total_slots: usize) -> Self {
        assert!(slot < total_slots.try_into().unwrap());
        match mode {
            0 => Self::new_normal_click(button, slot),
            // Both buttons do the same here, so we omit it
            1 => Self::new_shift_click(slot),
            2 => Self::new_key_click(button, slot),
            3 => Self {
                click_type: ClickType::CreativePickItem,
                slot: Slot::Normal(slot.try_into().unwrap()),
            },
            4 => Self::new_drop_item(button, slot),
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
                Slot::Normal(slot.try_into().unwrap())
            }
        };
        let button = match button {
            0 => Click::LeftClick,
            1 => Click::RightClick,
            // TODO: Error here
            _ => unreachable!(),
        };
        Self {
            click_type: ClickType::NormalClick(button),
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

    fn new_drop_item(button: i8, slot: i16) -> Self {
        let drop_type = match button {
            0 => DropType::SingleItem,
            1 => DropType::FullStack,
            // TODO: Error handling
            _ => unreachable!(),
        };
        Self {
            click_type: ClickType::DropType(drop_type),
            slot: Slot::Normal(slot.try_into().unwrap()),
        }
    }

    fn new_drag_item(button: i8, slot: i16) -> Self {
        let (mouse_type, state, slot) = match button {
            0 => (
                MouseDragType::LeftMouse,
                MouseDragState::Start,
                Slot::OutsideInventory,
            ),
            1 => (
                MouseDragType::LeftMouse,
                MouseDragState::AddSlot,
                Slot::Normal(slot.try_into().unwrap()),
            ),
            2 => (
                MouseDragType::LeftMouse,
                MouseDragState::End,
                Slot::OutsideInventory,
            ),

            4 => (
                MouseDragType::RightMouse,
                MouseDragState::Start,
                Slot::OutsideInventory,
            ),
            5 => (
                MouseDragType::RightMouse,
                MouseDragState::AddSlot,
                Slot::Normal(slot.try_into().unwrap()),
            ),
            6 => (
                MouseDragType::RightMouse,
                MouseDragState::End,
                Slot::OutsideInventory,
            ),

            // ONLY FOR CREATIVE
            8 => (
                MouseDragType::MiddleMouse,
                MouseDragState::Start,
                Slot::OutsideInventory,
            ),
            9 => (
                MouseDragType::MiddleMouse,
                MouseDragState::AddSlot,
                Slot::Normal(slot.try_into().unwrap()),
            ),
            10 => (
                MouseDragType::MiddleMouse,
                MouseDragState::End,
                Slot::OutsideInventory,
            ),
            // TODO: Error handling
            _ => unreachable!(),
        };
        Self {
            click_type: ClickType::MouseDrag {
                drag_state: state,
                drag_type: mouse_type,
            },
            slot,
        }
    }
}

enum ClickType {
    NormalClick(Click),
    ShiftClick,
    KeyClick(KeyClick),
    CreativePickItem,
    DropType(DropType),
    MouseDrag {
        drag_state: MouseDragState,
        drag_type: MouseDragType,
    },
    DoubleClick,
}
enum Click {
    LeftClick,
    RightClick,
    MiddleClick,
}

enum KeyClick {
    Slot(u8),
    Offhand,
}

enum Slot {
    Normal(usize),
    OutsideInventory,
}

enum DropType {
    SingleItem,
    FullStack,
}

enum MouseDragType {
    LeftMouse,
    RightMouse,
    MiddleMouse,
}

enum MouseDragState {
    Start,
    AddSlot,
    End,
}
