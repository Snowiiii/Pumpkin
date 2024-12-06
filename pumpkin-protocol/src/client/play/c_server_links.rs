use crate::VarInt;
use pumpkin_core::text::TextComponent;
use pumpkin_macros::client_packet;
use serde::{Serialize, Serializer};

#[derive(Serialize)]
#[client_packet("play:server_links")]
pub struct CServerLinks<'a> {
    links_count: &'a VarInt,
    links: &'a [Link<'a>],
}

impl<'a> CServerLinks<'a> {
    pub fn new(links_count: &'a VarInt, links: &'a [Link<'a>]) -> Self {
        Self { links_count, links }
    }
}

pub enum Label<'a> {
    BuiltIn(LinkType),
    TextComponent(TextComponent<'a>),
}

impl Serialize for Label<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Label::BuiltIn(link_type) => link_type.serialize(serializer),
            Label::TextComponent(component) => component.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
pub struct Link<'a> {
    pub is_built_in: bool,
    pub label: Label<'a>,
    pub url: &'a String,
}

impl<'a> Link<'a> {
    pub fn new(label: Label<'a>, url: &'a String) -> Self {
        Self {
            is_built_in: match label {
                Label::BuiltIn(_) => true,
                Label::TextComponent(_) => false,
            },
            label,
            url,
        }
    }
}

pub enum LinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl Serialize for LinkType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LinkType::BugReport => VarInt(0).serialize(serializer),
            LinkType::CommunityGuidelines => VarInt(1).serialize(serializer),
            LinkType::Support => VarInt(2).serialize(serializer),
            LinkType::Status => VarInt(3).serialize(serializer),
            LinkType::Feedback => VarInt(4).serialize(serializer),
            LinkType::Community => VarInt(5).serialize(serializer),
            LinkType::Website => VarInt(6).serialize(serializer),
            LinkType::Forums => VarInt(7).serialize(serializer),
            LinkType::News => VarInt(8).serialize(serializer),
            LinkType::Announcements => VarInt(9).serialize(serializer),
        }
    }
}
