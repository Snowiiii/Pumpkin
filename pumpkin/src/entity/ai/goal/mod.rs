pub mod look_at_entity;

pub trait Goal {
    /// How Should the Goal initially start?
    fn can_start(&self) -> bool;
    /// When its started, How it should Continue to run
    fn should_continue() -> bool;
    /// If the Goal is running, this gets called every tick
    fn tick(&self);
}
