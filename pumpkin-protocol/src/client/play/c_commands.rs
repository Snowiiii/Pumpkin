use bytes::{BufMut, BytesMut};
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, VarInt};

#[client_packet("play:commands")]
pub struct CCommands<'a> {
    pub nodes: Vec<ProtoNode<'a>>,
    pub root_node_index: VarInt,
}

impl<'a> CCommands<'a> {
    pub fn new(nodes: Vec<ProtoNode<'a>>, root_node_index: VarInt) -> Self {
        Self {
            nodes,
            root_node_index,
        }
    }
}

impl ClientPacket for CCommands<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_list(&self.nodes, |bytebuf, node: &ProtoNode| {
            node.write_to(bytebuf)
        });
        bytebuf.put_var_int(&self.root_node_index);
    }
}

pub struct ProtoNode<'a> {
    pub children: Vec<VarInt>,
    pub node_type: ProtoNodeType<'a>,
}

#[derive(Debug)]
pub enum ProtoNodeType<'a> {
    Root,
    Literal {
        name: &'a str,
        is_executable: bool,
    },
    Argument {
        name: &'a str,
        is_executable: bool,
        parser: ProtoCmdArgParser<'a>,
        override_suggestion_type: Option<ProtoCmdArgSuggestionType>,
    },
}

impl ProtoNode<'_> {
    const FLAG_IS_EXECUTABLE: u8 = 4;
    const FLAG_HAS_REDIRECT: u8 = 8;
    const FLAG_HAS_SUGGESTION_TYPE: u8 = 16;

    pub fn write_to(&self, bytebuf: &mut BytesMut) {
        // flags
        let flags = match self.node_type {
            ProtoNodeType::Root => 0,
            ProtoNodeType::Literal {
                name: _,
                is_executable,
            } => {
                let mut n = 1;
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            }
            ProtoNodeType::Argument {
                name: _,
                is_executable,
                parser: _,
                override_suggestion_type,
            } => {
                let mut n = 2;
                if override_suggestion_type.is_some() {
                    n |= Self::FLAG_HAS_SUGGESTION_TYPE
                }
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            }
        };
        bytebuf.put_u8(flags);

        // child count + children
        bytebuf.put_list(&self.children, |bytebuf, child| bytebuf.put_var_int(child));

        // redirect node
        if flags & Self::FLAG_HAS_REDIRECT != 0 {
            bytebuf.put_var_int(&1.into());
        }

        // name
        match self.node_type {
            ProtoNodeType::Argument { name, .. } | ProtoNodeType::Literal { name, .. } => {
                bytebuf.put_string(name)
            }
            ProtoNodeType::Root => {}
        }

        // parser id + properties
        if let ProtoNodeType::Argument {
            name: _,
            is_executable: _,
            parser,
            override_suggestion_type: _,
        } = &self.node_type
        {
            parser.write_to_buffer(bytebuf)
        }

        if flags & Self::FLAG_HAS_SUGGESTION_TYPE != 0 {
            match &self.node_type {
                ProtoNodeType::Argument { name: _, is_executable: _, parser: _, override_suggestion_type } => {
                    // suggestion type
                    let suggestion_type = &override_suggestion_type.expect("ProtoNode::FLAG_HAS_SUGGESTION_TYPE should only be set if override_suggestion_type is not None.");
                    bytebuf.put_string(suggestion_type.identifier());
                },
                _ => unimplemented!("ProtoNode::FLAG_HAS_SUGGESTION_TYPE is only implemented for ProtoNodeType::Argument"),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProtoCmdArgParser<'a> {
    Bool,
    Float { min: Option<f32>, max: Option<f32> },
    Double { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i32>, max: Option<i32> },
    Long { min: Option<i64>, max: Option<i64> },
    String(StringProtoArgBehavior),
    Entity { flags: u8 },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    Component,
    Style,
    Message,
    Nbt,
    NbtTag,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Angle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder { flags: u8 },
    Swizzle,
    Team,
    ItemSlot,
    ItemSlots,
    ResourceLocation,
    Function,
    EntityAnchor,
    IntRange,
    FloatRange,
    Dimension,
    Gamemode,
    Time { min: i32 },
    ResourceOrTag { identifier: &'a str },
    ResourceOrTagKey { identifier: &'a str },
    Resource { identifier: &'a str },
    ResourceKey { identifier: &'a str },
    TemplateMirror,
    TemplateRotation,
    Heightmap,
    LootTable,
    LootPredicate,
    LootModifier,
    Uuid,
}

impl ProtoCmdArgParser<'_> {
    pub const ENTITY_FLAG_ONLY_SINGLE: u8 = 1;
    pub const ENTITY_FLAG_PLAYERS_ONLY: u8 = 2;

    pub const SCORE_HOLDER_FLAG_ALLOW_MULTIPLE: u8 = 1;

    pub fn write_to_buffer(&self, bytebuf: &mut BytesMut) {
        match self {
            Self::Bool => bytebuf.put_var_int(&0.into()),
            Self::Float { min, max } => Self::write_number_arg(&1.into(), *min, *max, bytebuf),
            Self::Double { min, max } => Self::write_number_arg(&2.into(), *min, *max, bytebuf),
            Self::Integer { min, max } => Self::write_number_arg(&3.into(), *min, *max, bytebuf),
            Self::Long { min, max } => Self::write_number_arg(&4.into(), *min, *max, bytebuf),
            Self::String(behavior) => {
                bytebuf.put_var_int(&5.into());
                let i = match behavior {
                    StringProtoArgBehavior::SingleWord => 0,
                    StringProtoArgBehavior::QuotablePhrase => 1,
                    StringProtoArgBehavior::GreedyPhrase => 2,
                };
                bytebuf.put_var_int(&i.into());
            }
            Self::Entity { flags } => Self::write_with_flags(&6.into(), *flags, bytebuf),
            Self::GameProfile => bytebuf.put_var_int(&7.into()),
            Self::BlockPos => bytebuf.put_var_int(&8.into()),
            Self::ColumnPos => bytebuf.put_var_int(&9.into()),
            Self::Vec3 => bytebuf.put_var_int(&10.into()),
            Self::Vec2 => bytebuf.put_var_int(&11.into()),
            Self::BlockState => bytebuf.put_var_int(&12.into()),
            Self::BlockPredicate => bytebuf.put_var_int(&13.into()),
            Self::ItemStack => bytebuf.put_var_int(&14.into()),
            Self::ItemPredicate => bytebuf.put_var_int(&15.into()),
            Self::Color => bytebuf.put_var_int(&16.into()),
            Self::Component => bytebuf.put_var_int(&17.into()),
            Self::Style => bytebuf.put_var_int(&18.into()),
            Self::Message => bytebuf.put_var_int(&19.into()),
            Self::Nbt => bytebuf.put_var_int(&20.into()),
            Self::NbtTag => bytebuf.put_var_int(&21.into()),
            Self::NbtPath => bytebuf.put_var_int(&22.into()),
            Self::Objective => bytebuf.put_var_int(&23.into()),
            Self::ObjectiveCriteria => bytebuf.put_var_int(&24.into()),
            Self::Operation => bytebuf.put_var_int(&25.into()),
            Self::Particle => bytebuf.put_var_int(&26.into()),
            Self::Angle => bytebuf.put_var_int(&27.into()),
            Self::Rotation => bytebuf.put_var_int(&28.into()),
            Self::ScoreboardSlot => bytebuf.put_var_int(&29.into()),
            Self::ScoreHolder { flags } => Self::write_with_flags(&30.into(), *flags, bytebuf),
            Self::Swizzle => bytebuf.put_var_int(&31.into()),
            Self::Team => bytebuf.put_var_int(&32.into()),
            Self::ItemSlot => bytebuf.put_var_int(&33.into()),
            Self::ItemSlots => bytebuf.put_var_int(&34.into()),
            Self::ResourceLocation => bytebuf.put_var_int(&35.into()),
            Self::Function => bytebuf.put_var_int(&36.into()),
            Self::EntityAnchor => bytebuf.put_var_int(&37.into()),
            Self::IntRange => bytebuf.put_var_int(&38.into()),
            Self::FloatRange => bytebuf.put_var_int(&39.into()),
            Self::Dimension => bytebuf.put_var_int(&40.into()),
            Self::Gamemode => bytebuf.put_var_int(&41.into()),
            Self::Time { min } => {
                bytebuf.put_var_int(&42.into());
                bytebuf.put_i32(*min);
            }
            Self::ResourceOrTag { identifier } => {
                Self::write_with_identifier(&43.into(), identifier, bytebuf)
            }
            Self::ResourceOrTagKey { identifier } => {
                Self::write_with_identifier(&44.into(), identifier, bytebuf)
            }
            Self::Resource { identifier } => {
                Self::write_with_identifier(&45.into(), identifier, bytebuf)
            }
            Self::ResourceKey { identifier } => {
                Self::write_with_identifier(&46.into(), identifier, bytebuf)
            }
            Self::TemplateMirror => bytebuf.put_var_int(&47.into()),
            Self::TemplateRotation => bytebuf.put_var_int(&48.into()),
            Self::Heightmap => bytebuf.put_var_int(&49.into()),
            Self::LootTable => bytebuf.put_var_int(&50.into()),
            Self::LootPredicate => bytebuf.put_var_int(&51.into()),
            Self::LootModifier => bytebuf.put_var_int(&52.into()),
            Self::Uuid => bytebuf.put_var_int(&53.into()),
        }
    }

    fn write_number_arg<T: NumberCmdArg>(
        id: &VarInt,
        min: Option<T>,
        max: Option<T>,
        bytebuf: &mut BytesMut,
    ) {
        let mut flags: u8 = 0;
        if min.is_some() {
            flags |= 1
        }
        if max.is_some() {
            flags |= 2
        }

        bytebuf.put_var_int(id);

        bytebuf.put_u8(flags);
        if let Some(min) = min {
            min.write(bytebuf);
        }
        if let Some(max) = max {
            max.write(bytebuf);
        }
    }

    fn write_with_flags(id: &VarInt, flags: u8, bytebuf: &mut BytesMut) {
        bytebuf.put_var_int(id);

        bytebuf.put_u8(flags);
    }

    fn write_with_identifier(id: &VarInt, extra_identifier: &str, bytebuf: &mut BytesMut) {
        bytebuf.put_var_int(id);

        bytebuf.put_string(extra_identifier);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StringProtoArgBehavior {
    SingleWord,
    QuotablePhrase,
    /// does not stop after a space
    GreedyPhrase,
}

trait NumberCmdArg {
    fn write(self, bytebuf: &mut BytesMut);
}

impl NumberCmdArg for f32 {
    fn write(self, bytebuf: &mut BytesMut) {
        bytebuf.put_f32(self);
    }
}

impl NumberCmdArg for f64 {
    fn write(self, bytebuf: &mut BytesMut) {
        bytebuf.put_f64(self);
    }
}

impl NumberCmdArg for i32 {
    fn write(self, bytebuf: &mut BytesMut) {
        bytebuf.put_i32(self);
    }
}

impl NumberCmdArg for i64 {
    fn write(self, bytebuf: &mut BytesMut) {
        bytebuf.put_i64(self);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProtoCmdArgSuggestionType {
    AskServer,
    AllRecipes,
    AvailableSounds,
    SummonableEntities,
}

impl ProtoCmdArgSuggestionType {
    fn identifier(&self) -> &'static str {
        match self {
            Self::AskServer => "minecraft:ask_server",
            Self::AllRecipes => "minecraft:all_recipes",
            Self::AvailableSounds => "minecraft:available_sounds",
            Self::SummonableEntities => "minecraft:summonable_entities",
        }
    }
}
