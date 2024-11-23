use std::sync::LazyLock;

use proc_macro::TokenStream;
use serde::Deserialize;


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

#[expect(dead_code)]
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

static SLICE: [usize; 3] = [0, 1, 2];

static TEST: &'static [usize] = &[0, 1, 2];


static BLOCKS: LazyLock<JsonTopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

pub(crate) fn blocks_impl(_item: TokenStream) -> TokenStream {

    let mut max_collision_shapes: usize = 0;
    let mut max_states: usize = 0;
    let mut max_properties: usize = 0;
    let mut max_property_values: usize = 0;

    for block in &BLOCKS.blocks {
        if block.states.len() > max_states {
            max_states = block.states.len();
        }

        if block.properties.len() > max_properties {
            max_properties = block.properties.len();
        }
        
        for property in &block.properties {
            if property.values.len() > max_property_values {
                max_property_values = property.values.len();
            } 
        }

        for state in &block.states {
            if state.collision_shapes.len() > max_collision_shapes {
                max_collision_shapes = state.collision_shapes.len();
            }
        }
    }

    let mut blocks = Vec::with_capacity(BLOCKS.blocks.len());
    for block in &BLOCKS.blocks {

        let id = block.id;
        let item_id = block.item_id;
        let hardness = block.hardness;
        let wall_variant_id = match block.wall_variant_id {
            Some(v) => quote::quote! { Some( #v ) },
            None => quote::quote! { None },
        };
        let translation_key = &block.translation_key;
        let name = &block.name;
        let properties = block.properties.clone(); // TODO
        let default_state_id = block.default_state_id;
        let states = block.states.clone(); // TODO

        let tokens = quote::quote! {
            Block {
                id: #id,
                item_id: #item_id,
                hardness: #hardness,
                wall_variant_id: #wall_variant_id,
                translation_key: #translation_key,
                name: #name,
                properties: [],
                default_state_id: #default_state_id,
                states: [],
            }
        };

        blocks.push(tokens);
    }

    let block_count = blocks.len();

    let quote = quote::quote! { 
        pub type Block = pumpkin_core::registries::blocks::GenericBlock<
            #max_collision_shapes,
            #max_states,
            #max_properties,
            #max_property_values,
        >;

        pub static BLOCK: [Block; #block_count ] = [ #(#blocks),* ];
    };
    dbg!(quote.to_string());

    quote.into()
}

/// TODO
pub(crate) fn block_states_impl(_item: TokenStream) -> TokenStream {
    quote::quote! {}.into()
}