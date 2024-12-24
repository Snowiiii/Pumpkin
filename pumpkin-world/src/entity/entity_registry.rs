use std::{collections::HashMap, sync::LazyLock};

const ENTITIES_JSON: &str = include_str!("../../../assets/entities.json");

pub static ENTITIES: LazyLock<Vec<String>> = LazyLock::new(|| {
    serde_json::from_str(ENTITIES_JSON).expect("Could not parse entity.json registry.")
});

pub static ENTITIES_BY_ID: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, entity_name) in ENTITIES.iter().enumerate() {
        map.insert(entity_name.clone(), i as u16);
    }
    map
});

pub fn get_entity_id(name: &str) -> Option<&u16> {
    ENTITIES_BY_ID.get(&name.replace("minecraft:", ""))
}
