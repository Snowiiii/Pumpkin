use std::sync::LazyLock;

use proc_macro::TokenStream;
use serde::Deserialize;

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct JsonTopLevel {
    pub blocks: Vec<JsonBlock>,
    shapes: Vec<JsonShape>,
    block_entity_types: Vec<JsonBlockEntityKind>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JsonBlock {
    pub id: u16,
    pub item_id: u16,
    pub hardness: f32,
    pub wall_variant_id: Option<u16>,
    pub translation_key: String,
    pub name: String,
    pub properties: Vec<JsonBlockProperty>,
    pub default_state_id: u16,
    pub states: Vec<JsonBlockState>,
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct JsonBlockEntityKind {
    id: u32,
    ident: String,
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JsonBlockProperty {
    name: String,
    values: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JsonBlockState {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub collision_shapes: Vec<u16>,
    pub block_entity_type: Option<u32>,
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct JsonShape {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

static BLOCKS: LazyLock<JsonTopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

pub(crate) fn include_blocks_impl(_item: TokenStream) -> TokenStream {

    let mut blocks = Vec::with_capacity(BLOCKS.blocks.len());
    let mut all_states = Vec::new();

    for block in &BLOCKS.blocks {

        let id = block.id;
        let item_id = block.item_id;
        let hardness = block.hardness;
        let translation_key = &block.translation_key;
        let name = &block.name;
        let default_state_id = block.default_state_id;

        let wall_variant_id = match block.wall_variant_id {
            Some(v) => quote::quote! { Some( #v ) },
            None => quote::quote! { None },
        };

        let properties = block.properties.iter().map(|p| {

            let p_name = &p.name;
            let p_values = &p.values;

            quote::quote! {
                pumpkin_core::registries::blocks::Property {
                    name: #p_name,
                    values: &[ #(#p_values),* ],
                }
            }
        });

        let state_indices = block.states.iter().map(|s| {

            let s_id = s.id;
            let s_air = s.air;
            let s_luminance = s.luminance;
            let s_burnable = s.burnable;
            let s_opacity = match s.opacity {
                Some(v) => quote::quote! { Some( #v ) },
                None => quote::quote! { None },
            };
            let s_replaceable = s.replaceable;
            let s_collision_shapes = &s.collision_shapes;
            let s_block_entity_type = match s.block_entity_type {
                Some(v) => quote::quote! { Some( #v ) },
                None => quote::quote! { None },
            };

            let state_tokens = quote::quote! {
                pumpkin_core::registries::blocks::State {
                    id: #s_id,
                    block_id: #id,
                    air: #s_air,
                    luminance: #s_luminance,
                    burnable: #s_burnable,
                    opacity: #s_opacity,
                    replaceable: #s_replaceable,
                    collision_shapes: &[ #(#s_collision_shapes),* ],
                    block_entity_type: #s_block_entity_type,
                }
            };

            all_states.push(state_tokens);
            s.id
        });

        let block_tokens = quote::quote! {
            pumpkin_core::registries::blocks::Block {
                id: #id,
                item_id: #item_id,
                hardness: #hardness,
                wall_variant_id: #wall_variant_id,
                translation_key: #translation_key,
                name: #name,
                properties: &[ #(#properties),* ],
                default_state_id: #default_state_id,
                states: &[ #(#state_indices),* ],
            }
        };

        blocks.push(block_tokens);
    }

    let block_count = blocks.len();
    let state_count = all_states.len();

    let quote = quote::quote! { 
        pub static BLOCKS: [pumpkin_core::registries::blocks::Block; #block_count ] = [ #(#blocks),* ];
        pub static BLOCK_STATES: [pumpkin_core::registries::blocks::State; #state_count ] = [ #(#all_states),* ];
    };

    quote.into()
}