use super::block_registry::{get_block, get_state_by_state_id};

#[derive(Clone, Copy, Debug, Eq)]
pub struct BlockState {
    pub state_id: u16,
    pub block_id: u16,
}

impl PartialEq for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.state_id == other.state_id
    }
}

impl BlockState {
    pub const AIR: BlockState = BlockState {
        state_id: 0,
        block_id: 0,
    };

    /// Get a Block from the Vanilla Block registry at Runtime
    pub fn new(registry_id: &str) -> Option<Self> {
        let block = get_block(registry_id);
        block.map(|block| Self {
            state_id: block.default_state_id,
            block_id: block.id,
        })
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }

    #[inline]
    pub fn is_air(&self) -> bool {
        get_state_by_state_id(self.state_id).unwrap().air
    }

    #[inline]
    pub fn of_block(&self, block_id: u16) -> bool {
        self.block_id == block_id
    }
}

#[cfg(test)]
mod tests {
    use super::BlockState;

    #[test]
    fn not_existing() {
        let result = BlockState::new("this_block_does_not_exist");
        assert!(result.is_none());
    }

    #[test]
    fn does_exist() {
        let result = BlockState::new("minecraft:dirt");
        assert!(result.is_some());
    }
}
