use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

const ENTITIES_JSON: &str = include_str!("../../../assets/entities.json");

pub static ENTITIES: LazyLock<HashMap<String, Entity>> = LazyLock::new(|| {
    serde_json::from_str(ENTITIES_JSON).expect("Could not parse entity.json registry.")
});

pub static ENTITIES_BY_ID: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (entity_name, entity) in ENTITIES.iter() {
        map.insert(entity_name.clone(), entity.id);
    }
    map
});

pub fn get_entity_id(name: &str) -> Option<&u16> {
    ENTITIES_BY_ID.get(&name.replace("minecraft:", ""))
}

pub fn get_entity_by_id<'a>(entity_id: u16) -> Option<&'a Entity> {
    ENTITIES.values().find(|&entity| entity.id == entity_id)
}

#[derive(Deserialize, Clone, Debug)]
pub struct Entity {
    pub id: u16,
    pub max_health: Option<f64>,
    pub attackable: bool,
    pub summonable: bool,
    pub fire_immune: bool,
    pub dimension: Vec<f32>,
}
