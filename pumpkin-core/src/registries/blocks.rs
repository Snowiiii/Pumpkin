#[derive(Debug)]
pub struct Block {
    pub id: u16,
    pub item_id: u16,
    pub hardness: f32,
    pub wall_variant_id: Option<u16>,
    pub translation_key: &'static str,
    pub name: &'static str,
    pub properties: &'static [Property],
    pub default_state_id: u16,
    pub states: &'static [u16],
}

#[expect(dead_code)]
#[derive(Debug)]
struct BlockEntityKind {
    pub id: u32,
    pub ident: &'static str,
    pub name: &'static str,
}

#[derive(Debug)]
pub struct Property {
    pub name: &'static str,
    pub values: &'static [&'static str],
}

#[derive(Debug)]
pub struct State {
    pub id: u16,
    pub block_id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub collision_shapes: &'static [u16],
    pub block_entity_type: Option<u32>,
}

#[expect(dead_code)]
#[derive(Debug)]
struct Shape {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}