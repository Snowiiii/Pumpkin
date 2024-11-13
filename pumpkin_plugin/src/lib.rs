use pumpkin::{plugin::PumpkinPlugin, register_plugin};

#[derive(Debug, Default)]
struct MyPumpkinPlugin;

impl PumpkinPlugin for MyPumpkinPlugin {
    fn init(&mut self) {
        println!("Hello from MyPumpkinPlugin");
    }
}

register_plugin!(MyPumpkinPlugin);
