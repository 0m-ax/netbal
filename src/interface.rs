use crate::packet::OuterPacket;
use std::net::SocketAddr;
use std::time::Duration;
use mio::Token;

pub struct Interface{
    pub weight:u8,
    pub iid:u8,
    pub upstream:SocketAddr,
    pub last_rx:Duration,
    pub last_tx:Duration,
    pub name:String,
    pub socket_token:Token,
    pub packet_id:u16,
    pub is_static:bool,
    pub client_id:u16
}
impl Interface {
    pub fn new(weight: u8,iid: u8,upstream: SocketAddr, name:String, socket_token:Token,is_static:bool,client_id:u16) -> Self{
        Self{
            weight,
            iid,
            upstream,
            last_rx:Duration::new(0, 0),
            last_tx:Duration::new(0, 0),
            name:name,
            socket_token,
            packet_id:0,
            is_static,
            client_id
        }
    }
    pub fn make_packet(&mut self,data_packet: bool,data: Vec<u8>) -> OuterPacket  {
        self.inc_pid();
        OuterPacket {
            data_packet: data_packet,
            pid: self.packet_id,
            cid: self.client_id,
            iid: self.iid,
            data:data,
        }
    }
    fn inc_pid(&mut self){
        self.packet_id = self.packet_id+1;
        if self.packet_id > 1023 {
            self.packet_id = 0;
        }
    }
}