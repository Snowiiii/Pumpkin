use serde::Deserialize;

mod item;

pub use item::ITEM_TAGS;
#[derive(Deserialize)]
pub struct Tag {
    values: Vec<String>,
}
