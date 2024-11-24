use std::env;
use std::path::Path;

use serde::Deserialize;
use std::io::Write;

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
    pub first_state_id: u16,
    pub last_state_id: u16,
}

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
    pub block_id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub block_entity_type: Option<u32>,
}

#[derive(Deserialize, Clone, Debug)]
struct JsonShape {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

fn main() {
    println!("cargo::rerun-if-changed=../assets");

    generate_blocks();
    generate_block_states();
    generate_block_entities();
    generate_shapes();
    generate_block_state_properties();
    generate_block_state_collision_shapes();
}

fn generate_block_state_collision_shapes() {
    let json_data: Vec<Vec<u16>> = serde_json::from_str(include_str!("../assets/block_state_collision_shapes.json"))
        .expect("Could not parse json file.");

    let shapes: Vec<_> = json_data.into_iter().map(|block_state_shape_idxs| quote::quote! {
        &[ #(#block_state_shape_idxs),* ]
    }).collect();

    let len: usize = shapes.len();
    save_to_file("block_state_collision_shapes.rs", quote::quote! {
        pub static BLOCK_STATE_COLLISION_SHAPES: [&[u16]; #len ] = [ #(#shapes),* ];
    });
}

fn generate_shapes() {
    let json_data: Vec<JsonShape> = serde_json::from_str(include_str!("../assets/block_shapes.json"))
        .expect("Could not parse json file.");

    let shapes: Vec<_> = json_data.into_iter().map(|s| {
        let (max_x, max_y, max_z) = (s.max_x, s.max_y, s.max_z);
        let (min_x, min_y, min_z) = (s.min_x, s.min_y, s.min_z);
        quote::quote! {
            Shape {
                max_x: #max_x,
                max_y: #max_y,
                max_z: #max_z,
                min_x: #min_x,
                min_y: #min_y,
                min_z: #min_z,
            }
        }
    }).collect();

    let len: usize = shapes.len();
    save_to_file("block_shapes.rs", quote::quote! {
        use pumpkin_core::registries::blocks::Shape;
        pub static BLOCK_SHAPES: [Shape; #len ] = [ #(#shapes),* ];
    });
}

fn generate_block_state_properties() {
    let json_data: Vec<Vec<&str>> = serde_json::from_str(include_str!("../assets/block_state_properties.json"))
        .expect("Could not parse json file.");

    let properties: Vec<_> = json_data.into_iter().map(|block_state_property_values| quote::quote! {
        &[ #(#block_state_property_values),* ]
    }).collect();

    let len = properties.len();
    save_to_file("block_state_properties.rs", quote::quote! {
        pub static BLOCK_STATE_PROPERTY_VALUES: [&[&str]; #len ] = [ #(#properties),* ];
    });
}

fn generate_block_entities() {
    let json_data: Vec<JsonBlockEntityKind> = serde_json::from_str(include_str!("../assets/block_entities.json"))
        .expect("Could not parse json file.");

    let block_entities: Vec<_> = json_data.into_iter().map(|e| {
        let id = e.id;
        let name = e.name;
        let ident = e.ident;
        quote::quote! {
            BlockEntityKind {
                id: #id,
                name: #name,
                ident: #ident,
            }
        }
    }).collect();

    let len: usize = block_entities.len();
    save_to_file("block_entities.rs", quote::quote! {
        use pumpkin_core::registries::blocks::BlockEntityKind;
        pub static BLOCK_ENTITY_KINDS: [BlockEntityKind; #len ] = [ #(#block_entities),* ];
    });
}

fn generate_block_states() {
    let json_data: Vec<JsonBlockState> = serde_json::from_str(include_str!("../assets/block_states.json"))
        .expect("Could not parse json file.");

    let states: Vec<_> = json_data.into_iter().map(|s| {
        let (id, block_id) = (s.id, s.block_id);
        let (air, replacable) = (s.air, s.replaceable);
        let luminance = s.luminance;
        let burnable = s.burnable;
        let opacity = to_option_tokens(s.opacity);
        let block_entity = to_option_tokens(s.block_entity_type);
        quote::quote! {
            State {
                id: #id,
                block_id: #block_id,
                air: #air,
                replaceable: #replacable,
                luminance: #luminance,
                burnable: #burnable,
                opacity: #opacity,
                block_entity_type: #block_entity,
            }
        }
    }).collect();

    let len: usize = states.len();
    save_to_file("block_states.rs", quote::quote! {
        use pumpkin_core::registries::blocks::State;
        pub static BLOCK_STATES: [State; #len ] = [ #(#states),* ];
    });
}

fn generate_blocks() {
    let json_data: Vec<JsonBlock> = serde_json::from_str(include_str!("../assets/blocks.json"))
        .expect("Could not parse json file.");

    let blocks: Vec<_> = json_data.into_iter().map(|b| {
        let id = b.id;
        let name = b.name;
        let translation_key = b.translation_key;
        let hardness = b.hardness;
        let item_id = b.item_id;
        let default_state_id = b.default_state_id;
        let first_state_id = b.first_state_id;
        let last_state_id = b.last_state_id;
        let wall_variant_id = to_option_tokens(b.wall_variant_id);

        let properties = b.properties.iter().map(|p| {
            let p_name = &p.name;
            let p_values = &p.values;

            quote::quote! {
                Property {
                    name: #p_name,
                    values: &[ #(#p_values),* ],
                }
            }
        });

        quote::quote! {
            Block {
                id: #id,
                item_id: #item_id,
                hardness: #hardness,
                wall_variant_id: #wall_variant_id,
                translation_key: #translation_key,
                name: #name,
                properties: &[ #(#properties),* ],
                default_state_id: #default_state_id,
                first_state_id: #first_state_id,
                last_state_id: #last_state_id,
            }
        }
    }).collect();

    let len = blocks.len();
    save_to_file("blocks.rs", quote::quote! {
        use pumpkin_core::registries::blocks::Block;
        use pumpkin_core::registries::blocks::Property;
        pub static BLOCKS: [Block; #len ] = [ #(#blocks),* ];
    });
}

fn save_to_file(name: &str, tokens: proc_macro2::TokenStream) {
    let out_dir_name = env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir_name);

    let mut file = std::fs::File::create(out_path.join(name))
        .expect("Failed to create file");

    let parsed = syn::parse2(tokens)
        .expect("Failed to parse TokenStream");

    let formatted = prettyplease::unparse(&parsed);

    file.write_all(formatted.as_bytes())
        .expect("Failed to write to file");
}

fn to_option_tokens<T: quote::ToTokens>(option: Option<T>) -> proc_macro2::TokenStream {
    match option {
        Some(v) => quote::quote! { Some( #v ) },
        None => quote::quote! { None },
    }
}
