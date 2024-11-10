use std::sync::LazyLock;

use crate::command::{args::arg_test_client_side_arg_parsers::ClientSideArgParserTester, tree_builder::{argument_default_name, literal, NonLeafNodeBuilder}};

use pumpkin_protocol::client::play::{ProtoCmdArgParser, StringProtoArgBehavior};

static BOOL: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Bool));
static FLOAT: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Float { min: Some(-10.0), max: Some(10.0) }));
static DOUBLE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Double { min: Some(-10.0), max: Some(10.0) }));
static INTEGER: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Integer { min: Some(-10), max: Some(10) }));
static LONG: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Long { min: Some(-10), max: Some(10) }));
static STRING: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)));
static ENTITY: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Entity { flags: 0 }));
static GAMEPROFILE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::GameProfile));
static BLOCKPOS: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::BlockPos));
static COLUMNPOS: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ColumnPos));
static VEC3: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Vec3));
static VEC2: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Vec2));
static BLOCKSTATE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::BlockState));
static BLOCKPREDICATE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::BlockPredicate));
static ITEMSTACK: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ItemStack));
static ITEMPREDICATE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ItemPredicate));
static COLOR: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Color));
static COMPONENT: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Component));
static STYLE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Style));
static MESSAGE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Message));
static NBT: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Nbt));
static NBTTAG: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::NbtTag));
static NBTPATH: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::NbtPath));
static OBJECTIVE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Objective));
static OBJECTIVECRITERIA: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ObjectiveCriteria));
static OPERATION: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Operation));
static PARTICLE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Particle));
static ANGLE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Angle));
static ROTATION: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Rotation));
static SCOREBOARDSLOT: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ScoreboardSlot));
static SCOREHOLDER: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ScoreHolder { flags: 0 }));
static SWIZZLE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Swizzle));
static TEAM: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Team));
static ITEMSLOT: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ItemSlot));
static CONTAINERSLOT: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ContainerSlot));
static RESOURCELOCATION: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceLocation));
static FUNCTION: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Function));
static ENTITYANCHOR: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::EntityAnchor));
static INTRANGE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::IntRange));
static FLOATRANGE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::FloatRange));
static DIMENSION: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Dimension));
static GAMEMODE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Gamemode));
static TIME: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Time { min: 0 }));
static RESOURCEORTAG: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceOrTag { identifier: "item" }));
static RESOURCEORTAGKEY: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceOrTagKey { identifier: "item" }));
static RESOURCE: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Resource { identifier: "item" }));
static RESOURCEKEY: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceKey { identifier: "item" }));
static TEMPLATEMIRROR: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::TemplateMirror));
static TEMPLATEROTATION: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::TemplateRotation));
static HEIGHTMAP: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Heightmap));
static UUID: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Uuid));

pub(super) fn pumpkin_test_client_side_arg_parsers() -> NonLeafNodeBuilder<'static> {
    literal("client_side_arg_parsers")
        .with_child(literal("Bool").with_child(argument_default_name(&*BOOL)))
        .with_child(literal("Float").with_child(argument_default_name(&*FLOAT)))
        .with_child(literal("Double").with_child(argument_default_name(&*DOUBLE)))
        .with_child(literal("Integer").with_child(argument_default_name(&*INTEGER)))
        .with_child(literal("Long").with_child(argument_default_name(&*LONG)))
        .with_child(literal("String").with_child(argument_default_name(&*STRING)))
        .with_child(literal("Entity").with_child(argument_default_name(&*ENTITY)))
        .with_child(literal("GameProfile").with_child(argument_default_name(&*GAMEPROFILE)))
        .with_child(literal("BlockPos").with_child(argument_default_name(&*BLOCKPOS)))
        .with_child(literal("ColumnPos").with_child(argument_default_name(&*COLUMNPOS)))
        .with_child(literal("Vec3").with_child(argument_default_name(&*VEC3)))
        .with_child(literal("Vec2").with_child(argument_default_name(&*VEC2)))
        .with_child(literal("BlockState").with_child(argument_default_name(&*BLOCKSTATE)))
        .with_child(literal("BlockPredicate").with_child(argument_default_name(&*BLOCKPREDICATE)))
        .with_child(literal("ItemStack").with_child(argument_default_name(&*ITEMSTACK)))
        .with_child(literal("ItemPredicate").with_child(argument_default_name(&*ITEMPREDICATE)))
        .with_child(literal("Color").with_child(argument_default_name(&*COLOR)))
        .with_child(literal("Component").with_child(argument_default_name(&*COMPONENT)))
        .with_child(literal("Style").with_child(argument_default_name(&*STYLE)))
        .with_child(literal("Message").with_child(argument_default_name(&*MESSAGE)))
        .with_child(literal("Nbt").with_child(argument_default_name(&*NBT)))
        .with_child(literal("NbtTag").with_child(argument_default_name(&*NBTTAG)))
        .with_child(literal("NbtPath").with_child(argument_default_name(&*NBTPATH)))
        .with_child(literal("Objective").with_child(argument_default_name(&*OBJECTIVE)))
        .with_child(literal("ObjectiveCriteria").with_child(argument_default_name(&*OBJECTIVECRITERIA)))
        .with_child(literal("Operation").with_child(argument_default_name(&*OPERATION)))
        .with_child(literal("Particle").with_child(argument_default_name(&*PARTICLE)))
        .with_child(literal("Angle").with_child(argument_default_name(&*ANGLE)))
        .with_child(literal("Rotation").with_child(argument_default_name(&*ROTATION)))
        .with_child(literal("ScoreboardSlot").with_child(argument_default_name(&*SCOREBOARDSLOT)))
        .with_child(literal("ScoreHolder").with_child(argument_default_name(&*SCOREHOLDER)))
        .with_child(literal("Swizzle").with_child(argument_default_name(&*SWIZZLE)))
        .with_child(literal("Team").with_child(argument_default_name(&*TEAM)))
        .with_child(literal("ItemSlot").with_child(argument_default_name(&*ITEMSLOT)))
        .with_child(literal("ContainerSlot").with_child(argument_default_name(&*CONTAINERSLOT)))
        .with_child(literal("ResourceLocation").with_child(argument_default_name(&*RESOURCELOCATION)))
        .with_child(literal("Function").with_child(argument_default_name(&*FUNCTION)))
        .with_child(literal("EntityAnchor").with_child(argument_default_name(&*ENTITYANCHOR)))
        .with_child(literal("IntRange").with_child(argument_default_name(&*INTRANGE)))
        .with_child(literal("FloatRange").with_child(argument_default_name(&*FLOATRANGE)))
        .with_child(literal("Dimension").with_child(argument_default_name(&*DIMENSION)))
        .with_child(literal("Gamemode").with_child(argument_default_name(&*GAMEMODE)))
        .with_child(literal("Time").with_child(argument_default_name(&*TIME)))
        .with_child(literal("ResourceOrTag").with_child(argument_default_name(&*RESOURCEORTAG)))
        .with_child(literal("ResourceOrTagKey").with_child(argument_default_name(&*RESOURCEORTAGKEY)))
        .with_child(literal("Resource").with_child(argument_default_name(&*RESOURCE)))
        .with_child(literal("ResourceKey").with_child(argument_default_name(&*RESOURCEKEY)))
        .with_child(literal("TemplateMirror").with_child(argument_default_name(&*TEMPLATEMIRROR)))
        .with_child(literal("TemplateRotation").with_child(argument_default_name(&*TEMPLATEROTATION)))
        .with_child(literal("Heightmap").with_child(argument_default_name(&*HEIGHTMAP)))
        .with_child(literal("Uuid").with_child(argument_default_name(&*UUID)))
}