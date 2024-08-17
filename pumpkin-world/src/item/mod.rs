mod item_registry;

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Item Raritiy
pub enum Raritiy {
    Common,
    UnCommon,
    Rare,
    Epic,
}
