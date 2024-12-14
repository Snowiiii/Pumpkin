use pumpkin_config::BasicConfiguration;
use schemars::schema_for;

fn main() {
    println!("Hello, world!");
    let schema = schema_for!(BasicConfiguration);
    std::fs::write(
        "../configuration.schema.json",
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
}
