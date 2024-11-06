use super::block_registry::get_block;

#[derive(Clone)]
pub struct BlockState {
    pub state_id: u16,
}

impl BlockState {
    pub const AIR: BlockState = BlockState { state_id: 0 };

    /// Get a Block from the Vanilla Block registry at Runtime
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
