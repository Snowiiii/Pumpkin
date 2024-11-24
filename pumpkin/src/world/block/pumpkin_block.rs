use crate::world::block::block_manager::BlockManager;

pub trait BlockMetadata {
    const NAMESPACE: &'static str;
    const ID: &'static str;
    fn name(&self) -> String {
        format!("{}:{}", Self::NAMESPACE, Self::ID)
    }
}

pub trait PumpkinBlock: BlockMetadata {
    fn register(self, block_manager: &mut BlockManager);
}
