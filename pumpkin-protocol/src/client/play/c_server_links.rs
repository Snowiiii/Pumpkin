use crate::{Link, VarInt};
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:server_links")]
pub struct CPlayServerLinks<'a> {
    links_count: &'a VarInt,
    links: &'a [Link<'a>],
}

impl<'a> CPlayServerLinks<'a> {
    pub fn new(links_count: &'a VarInt, links: &'a [Link<'a>]) -> Self {
        Self { links_count, links }
    }
}
