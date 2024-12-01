use super::block_registry::get_block;

#[derive(Clone)]
pub struct BlockState {
    pub state_id: u16,
}

impl BlockState {
    pub const AIR: BlockState = BlockState { state_id: 0 };

    /// Get a Block from the Vanilla Block registry at Runtime.
    /// If block name is known at compile time, use [`get_block_state`] macro instead.
    pub fn new(registry_id: &str) -> Option<Self> {
        let block = get_block(registry_id);
        block.map(|block| Self {
            state_id: block.default_state_id,
        })
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }
}

/// Get a [`BlockState`] from the Vanilla Block registry at compile time.
macro_rules! get_block_state {
    ($identifier:literal) => {
        crate::block::BlockState {
            state_id: pumpkin_macros::block_state_id!($identifier),
        }
    };
}

pub(crate) use get_block_state;

#[cfg(test)]
mod tests {
    use super::BlockState;

    #[test]
    fn not_existing() {
        let result = BlockState::new("minecraft:this_block_does_not_exist");
        assert!(result.is_none());
    }

    #[test]
    fn does_exist() {
        let result = BlockState::new("minecraft:dirt");
        assert!(result.is_some());
    }

    #[test]
    fn qualified_macro() {
        get_block_state!("minecraft:dirt");
    }

    #[test]
    fn unqualified_macro() {
        get_block_state!("dirt");
    }
}
