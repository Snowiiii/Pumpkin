use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

use proc_macro::TokenStream;
use pumpkin_data::block::BLOCKS;
use pumpkin_data::block_entities::BLOCK_ENTITY_KINDS;

static BLOCK_MAP: LazyLock<HashMap<String, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, block) in BLOCKS.iter().enumerate() {
        map.insert(block.name.to_string(), i);
    }
    map
});

static BLOCK_ENTITY_MAP: LazyLock<HashMap<String, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, block_entity) in BLOCK_ENTITY_KINDS.iter().enumerate() {
        map.insert(block_entity.ident.to_string(), i);
    }
    map
});

fn token_stream_to_identifier(item: TokenStream) -> String {
    let literal = item.to_string();
    let name = literal
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .expect("identifier must be a string literal");

    if name.contains(':') {
        name.to_string()
    } else {
        format!("minecraft:{name}")
    }
}

pub(super) fn block_id_impl(item: TokenStream) -> TokenStream {
    let identifier = token_stream_to_identifier(item);

    let id = BLOCK_MAP[&identifier];

    // integer literal without type information
    TokenStream::from_str(&format!("{id}")).unwrap()
}

pub(super) fn block_state_id_impl(item: TokenStream) -> TokenStream {
    // todo: allow passing properties like in "oak_log[axis=z]"
    let identifier = token_stream_to_identifier(item);

    let block_id = BLOCK_MAP[&identifier];
    let block = &BLOCKS[block_id];
    let id = block.default_state_id;

    // integer literal without type information
    TokenStream::from_str(&format!("{id}")).unwrap()
}

pub(super) fn block_entity_id_impl(item: TokenStream) -> TokenStream {
    let identifier = token_stream_to_identifier(item);

    let id = BLOCK_ENTITY_MAP[&identifier];

    // integer literal without type information
    TokenStream::from_str(&format!("{id}")).unwrap()
}
