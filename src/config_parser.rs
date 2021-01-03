use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;


#[derive(Serialize, Deserialize)]
pub struct Client {
    pub id: u16,
    pub external_socket:ExternalSocket,
    pub interfaces:Vec<Interface>,
    pub allow_discover:bool
}

#[derive(Serialize, Deserialize)]
pub struct Interface {
    pub upstream: String,
    pub weight:u8,
    pub id:u8,
    pub socket_id:usize,
    pub name: String
}
#[derive(Serialize, Deserialize)]
pub struct ExternalSocket {
    pub address: String,
    pub upstream: String,
    pub netns: Option<String>
}
#[derive(Serialize, Deserialize)]
pub struct NetbalSocket {
    pub address: String,
    pub id:usize,
    pub netns: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct Config{
    pub clients: Vec<Client>,
    pub sockets: Vec<NetbalSocket>
}


pub fn parse_config(filename:String) -> Result<Config,Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;

    let p: Config = serde_json::from_str(&contents)?;
    return Ok(p);
}
