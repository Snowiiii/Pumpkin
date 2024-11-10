use std::sync::LazyLock;

use crate::command::{args::arg_test_client_side_arg_parsers::ClientSideArgParserTester, tree_builder::{argument_default_name, literal, NonLeafNodeBuilder}};

use pumpkin_protocol::client::play::{ProtoCmdArgParser, StringProtoArgBehavior};

static Bool: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Bool));
static Float: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Float { min: Some(-10.0), max: Some(10.0) }));
static Double: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Double { min: Some(-10.0), max: Some(10.0) }));
static Integer: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Integer { min: Some(-10), max: Some(10) }));
static Long: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Long { min: Some(-10), max: Some(10) }));
static String: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::String(StringProtoArgBehavior::SingleWord)));
static Entity: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Entity { flags: 0 }));
static GameProfile: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::GameProfile));
static BlockPos: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::BlockPos));
static ColumnPos: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ColumnPos));
static Vec3: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Vec3));
static Vec2: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Vec2));
static BlockState: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::BlockState));
static BlockPredicate: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::BlockPredicate));
static ItemStack: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ItemStack));
static ItemPredicate: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ItemPredicate));
static Color: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Color));
static Component: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Component));
static Style: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Style));
static Message: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Message));
static Nbt: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Nbt));
static NbtTag: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::NbtTag));
static NbtPath: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::NbtPath));
static Objective: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Objective));
static ObjectiveCriteria: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ObjectiveCriteria));
static Operation: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Operation));
static Particle: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Particle));
static Angle: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Angle));
static Rotation: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Rotation));
static ScoreboardSlot: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ScoreboardSlot));
static ScoreHolder: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ScoreHolder { flags: 0 }));
static Swizzle: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Swizzle));
static Team: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Team));
static ItemSlot: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ItemSlot));
static ContainerSlot: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ContainerSlot));
static ResourceLocation: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceLocation));
static Function: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Function));
static EntityAnchor: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::EntityAnchor));
static IntRange: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::IntRange));
static FloatRange: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::FloatRange));
static Dimension: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Dimension));
static Gamemode: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Gamemode));
static Time: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Time { min: 0 }));
static ResourceOrTag: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceOrTag { identifier: "item" }));
static ResourceOrTagKey: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceOrTagKey { identifier: "item" }));
static Resource: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Resource { identifier: "item" }));
static ResourceKey: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::ResourceKey { identifier: "item" }));
static TemplateMirror: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::TemplateMirror));
static TemplateRotation: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::TemplateRotation));
static Heightmap: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Heightmap));
static Uuid: LazyLock<ClientSideArgParserTester<'static>> = LazyLock::new(|| ClientSideArgParserTester(ProtoCmdArgParser::Uuid));

pub(super) fn pumpkin_test_client_side_arg_parsers() -> NonLeafNodeBuilder<'static> {
    literal("client_side_arg_parsers")
        .with_child(literal("Bool").with_child(argument_default_name(&*Bool)))
        .with_child(literal("Float").with_child(argument_default_name(&*Float)))
        .with_child(literal("Double").with_child(argument_default_name(&*Double)))
        .with_child(literal("Integer").with_child(argument_default_name(&*Integer)))
        .with_child(literal("Long").with_child(argument_default_name(&*Long)))
        .with_child(literal("String").with_child(argument_default_name(&*String)))
        .with_child(literal("Entity").with_child(argument_default_name(&*Entity)))
        .with_child(literal("GameProfile").with_child(argument_default_name(&*GameProfile)))
        .with_child(literal("BlockPos").with_child(argument_default_name(&*BlockPos)))
        .with_child(literal("ColumnPos").with_child(argument_default_name(&*ColumnPos)))
        .with_child(literal("Vec3").with_child(argument_default_name(&*Vec3)))
        .with_child(literal("Vec2").with_child(argument_default_name(&*Vec2)))
        .with_child(literal("BlockState").with_child(argument_default_name(&*BlockState)))
        .with_child(literal("BlockPredicate").with_child(argument_default_name(&*BlockPredicate)))
        .with_child(literal("ItemStack").with_child(argument_default_name(&*ItemStack)))
        .with_child(literal("ItemPredicate").with_child(argument_default_name(&*ItemPredicate)))
        .with_child(literal("Color").with_child(argument_default_name(&*Color)))
        .with_child(literal("Component").with_child(argument_default_name(&*Component)))
        .with_child(literal("Style").with_child(argument_default_name(&*Style)))
        .with_child(literal("Message").with_child(argument_default_name(&*Message)))
        .with_child(literal("Nbt").with_child(argument_default_name(&*Nbt)))
        .with_child(literal("NbtTag").with_child(argument_default_name(&*NbtTag)))
        .with_child(literal("NbtPath").with_child(argument_default_name(&*NbtPath)))
        .with_child(literal("Objective").with_child(argument_default_name(&*Objective)))
        .with_child(literal("ObjectiveCriteria").with_child(argument_default_name(&*ObjectiveCriteria)))
        .with_child(literal("Operation").with_child(argument_default_name(&*Operation)))
        .with_child(literal("Particle").with_child(argument_default_name(&*Particle)))
        .with_child(literal("Angle").with_child(argument_default_name(&*Angle)))
        .with_child(literal("Rotation").with_child(argument_default_name(&*Rotation)))
        .with_child(literal("ScoreboardSlot").with_child(argument_default_name(&*ScoreboardSlot)))
        .with_child(literal("ScoreHolder").with_child(argument_default_name(&*ScoreHolder)))
        .with_child(literal("Swizzle").with_child(argument_default_name(&*Swizzle)))
        .with_child(literal("Team").with_child(argument_default_name(&*Team)))
        .with_child(literal("ItemSlot").with_child(argument_default_name(&*ItemSlot)))
        .with_child(literal("ContainerSlot").with_child(argument_default_name(&*ContainerSlot)))
        .with_child(literal("ResourceLocation").with_child(argument_default_name(&*ResourceLocation)))
        .with_child(literal("Function").with_child(argument_default_name(&*Function)))
        .with_child(literal("EntityAnchor").with_child(argument_default_name(&*EntityAnchor)))
        .with_child(literal("IntRange").with_child(argument_default_name(&*IntRange)))
        .with_child(literal("FloatRange").with_child(argument_default_name(&*FloatRange)))
        .with_child(literal("Dimension").with_child(argument_default_name(&*Dimension)))
        .with_child(literal("Gamemode").with_child(argument_default_name(&*Gamemode)))
        .with_child(literal("Time").with_child(argument_default_name(&*Time)))
        .with_child(literal("ResourceOrTag").with_child(argument_default_name(&*ResourceOrTag)))
        .with_child(literal("ResourceOrTagKey").with_child(argument_default_name(&*ResourceOrTagKey)))
        .with_child(literal("Resource").with_child(argument_default_name(&*Resource)))
        .with_child(literal("ResourceKey").with_child(argument_default_name(&*ResourceKey)))
        .with_child(literal("TemplateMirror").with_child(argument_default_name(&*TemplateMirror)))
        .with_child(literal("TemplateRotation").with_child(argument_default_name(&*TemplateRotation)))
        .with_child(literal("Heightmap").with_child(argument_default_name(&*Heightmap)))
        .with_child(literal("Uuid").with_child(argument_default_name(&*Uuid)))
}