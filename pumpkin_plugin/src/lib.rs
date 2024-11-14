use pumpkin::{
    plugin::{logging, EventRegistry, PumpkinPlugin},
    register_plugin,
};

#[derive(Debug, Default)]
struct MyPumpkinPlugin;

impl PumpkinPlugin for MyPumpkinPlugin {
    fn init(&mut self, event_registry: &mut EventRegistry) {
        logging::warn!("I am god");
        event_registry.register_on_init(|_| logging::debug!("On Init event called!"));
    }
}

register_plugin!(MyPumpkinPlugin);
