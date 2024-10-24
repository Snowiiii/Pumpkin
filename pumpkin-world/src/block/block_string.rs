use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct BlockString {
    pub name: &'static str,
    pub properties: &'static HashMap<String, String>,
}
