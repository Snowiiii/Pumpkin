#[derive(Debug)]
pub struct GenericBlock<
    const MAX_COLLISION_SHAPES: usize,
    const MAX_STATES: usize,
    const MAX_PROPERTIES: usize,
    const MAX_PROPERTY_VALUES: usize,
> {
    pub id: u16,
    pub item_id: u16,
    pub hardness: f32,
    pub wall_variant_id: Option<u16>,
    pub translation_key: &'static str,
    pub name: &'static str,
    pub properties: [Option<Property<MAX_PROPERTY_VALUES>>; MAX_PROPERTIES],
    pub default_state_id: u16,
    pub states: [Option<u16>; MAX_STATES],
}

#[expect(dead_code)]
#[derive(Debug)]
struct BlockEntityKind {
    id: u32,
    ident: &'static str,
    name: &'static str,
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct Property<const MAX_PROPERTY_VALUES: usize> {
    name: &'static str,
    values: [Option<&'static str>; MAX_PROPERTY_VALUES],
}

#[derive(Debug)]
pub struct State<const MAX_COLLISION_SHAPES: usize> {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub collision_shapes: [u16; MAX_COLLISION_SHAPES],
    pub block_entity_type: Option<u32>,
}

#[expect(dead_code)]
#[derive(Debug)]
struct Shape {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}