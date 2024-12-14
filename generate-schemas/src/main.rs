use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use schemars::schema_for;

fn main() {
    let basic_schema = schema_for!(BasicConfiguration);
    let advanced_schema = schema_for!(AdvancedConfiguration);
    std::fs::write(
        "../configuration.schema.json",
        serde_json::to_string_pretty(&basic_schema)
            .map_err(|err| eprintln!("Couldn't generate basic schema: {}", err))
            .unwrap_or("{\"error\": \"Couldn't generate basic schema\"}".to_string()),
    )
    .map_err(|err| eprintln!("Couldn't write basic schema: {}", err))
    .unwrap();
    std::fs::write(
        "../features.schema.json",
        serde_json::to_string_pretty(&advanced_schema)
            .map_err(|err| eprintln!("Couldn't generate basic schema: {}", err))
            .unwrap_or("{\"error\": \"Couldn't generate basic schema\"}".to_string()),
    )
    .map_err(|err| eprintln!("Couldn't write advanced schema: {}", err))
    .unwrap();
}
