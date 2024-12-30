use crate::{Link, VarInt};
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("config:server_links")]
pub struct CConfigServerLinks<'a> {
    links_count: &'a VarInt,
    links: &'a [Link<'a>],
}

impl<'a> CConfigServerLinks<'a> {
    pub fn new(links_count: &'a VarInt, links: &'a [Link<'a>]) -> Self {
        Self { links_count, links }
    }
}
