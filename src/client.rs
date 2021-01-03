use std::net::SocketAddr;
use mio::Token;
use crate::interface::Interface;
use indexmap::IndexMap;

pub struct Client {
    pub id:u16,
    pub interfaces:IndexMap<u8,Interface>,
    pub external_socket:Token,
    pub external_upstream:SocketAddr,
    pub allow_discover:bool,
    pub next_interface_index:usize,
}