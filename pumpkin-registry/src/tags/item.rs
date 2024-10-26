use super::{Tag, TagType};
use std::collections::HashMap;
use std::sync::LazyLock;

pub static ITEM_TAGS: LazyLock<HashMap<String, Vec<TagType>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for tag in std::fs::read_dir("assets/tags/item").unwrap() {
        let r = tag.unwrap();
        let name = r.file_name().to_str().unwrap().to_string();
        if let Ok(s) = std::fs::read_to_string(r.path()) {
            let tag = serde_json::from_str::<Tag>(&s).unwrap();
            let name = name.strip_suffix(".json").unwrap();
            map.insert(name.to_string(), tag.values);
        }
    }
    map
});
