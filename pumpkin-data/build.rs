use std::env;
use std::path::Path;

use std::sync::LazyLock;

use serde::Deserialize;
use std::io::Write;

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
    serde_json::from_str(include_str!("../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

fn main() {
    println!("cargo::rerun-if-changed=../assets");

    let out_dir_name = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir_name);
    generate_blocks(out_path);
}

fn generate_blocks(dir: &Path) {
    let mut blocks = Vec::with_capacity(BLOCKS.blocks.len());
    let mut all_states = Vec::new();
    let mut all_state_collsion_shapes = Vec::new();

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
                Property {
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

            all_states.push(quote::quote! {
                State {
                    id: #s_id,
                    block_id: #id,
                    air: #s_air,
                    luminance: #s_luminance,
                    burnable: #s_burnable,
                    opacity: #s_opacity,
                    replaceable: #s_replaceable,
                    //collision_shapes: &[ #(#s_collision_shapes),* ],
                    block_entity_type: #s_block_entity_type,
                }
            });

            all_state_collsion_shapes.push(quote::quote! {
                &[ #(#s_collision_shapes),* ]
            });

            s.id
        });

        let block_tokens = quote::quote! {
            Block {
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

    // module for blocks
    let block_count = blocks.len();
    let block_tokens = quote::quote! {
        use pumpkin_core::registries::blocks::Block;
        use pumpkin_core::registries::blocks::Property;
        pub static BLOCKS: [Block; #block_count ] = [ #(#blocks),* ];
    };
    let mut block_file =
        std::fs::File::create(dir.join("block_data.rs")).expect("Failed to create file");
    block_file
        .write_all(
            prettyplease::unparse(&syn::parse2(block_tokens).expect("Failed to parse TokenStream"))
                .as_bytes(),
        )
        .expect("Failed to write to file");

    // module for block states
    let state_count = all_states.len();
    let state_tokens = quote::quote! {
        use pumpkin_core::registries::blocks::State;
        pub static BLOCK_STATES: [State; #state_count ] = [ #(#all_states),* ];
    };
    let mut state_file =
        std::fs::File::create(dir.join("state_data.rs")).expect("Failed to create file");
    state_file
        .write_all(
            prettyplease::unparse(&syn::parse2(state_tokens).expect("Failed to parse TokenStream"))
                .as_bytes(),
        )
        .expect("Failed to write to file");

    // seperate module for block state collision shapes so they can be compiled in parallel
    let state_collision_shape_tokens = quote::quote! {
        pub static BLOCK_STATE_COLLISION_SHAPES: [&'static [u16]; #state_count ] = [ #(#all_state_collsion_shapes),* ];
    };
    let mut state_collision_shape_file =
        std::fs::File::create(dir.join("state_collision_shape_data.rs"))
            .expect("Failed to create file");
    state_collision_shape_file
        .write_all(
            prettyplease::unparse(
                &syn::parse2(state_collision_shape_tokens).expect("Failed to parse TokenStream"),
            )
            .as_bytes(),
        )
        .expect("Failed to write to file");
}
