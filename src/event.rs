use mio::{Poll, Token};
use crate::socket::{ExternalSocket,NetbalSocket};
use mio_extras::timer::Timer;
use std::collections::HashMap;
use std::io;
use std::iter::Iterator;
use mio::*;


pub enum EventType<'a> {
    NetbalSocket(&'a mut NetbalSocket),
    ExternalSocket(&'a mut ExternalSocket),
    Timer(&'a mut Timer<String>),
    Unkown
}
pub struct EventManager {
    poll: Poll,
    type_map: HashMap<Token, usize>,
    external_map: HashMap<Token, ExternalSocket>,
    netbal_map: HashMap<Token, NetbalSocket>,
    timer_map: HashMap<Token, Timer<String>>,
    index: usize,
}
pub struct Events {
    events: mio::Events,
}


impl EventManager {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            poll: Poll::new()?,
            type_map: HashMap::new(),
            external_map: HashMap::new(),
            netbal_map: HashMap::new(),
            timer_map: HashMap::new(),
            index:0
        })
    }

    pub fn poll(&mut self) -> mio::Events  {
        let mut events = mio::Events::with_capacity(1024);
        match self.poll.poll(&mut events, None) {
            Ok(_) => (),
            Err(_) => (),
        };
        events
    }
    pub fn get(&mut self, token: Token) -> EventType  {
        match self.type_map.get(&token){
            Some(1) => match self.external_map.get_mut(&token){
                Some(socket) => EventType::ExternalSocket(socket),
                None => EventType::Unkown
            },
            Some(2) => match self.netbal_map.get_mut(&token){
                Some(socket) => EventType::NetbalSocket(socket),
                None => EventType::Unkown
            },
            Some(3) => match self.timer_map.get_mut(&token){
                Some(socket) => EventType::Timer(socket),
                None => EventType::Unkown
            },
            _ => EventType::Unkown
        }
    }
    pub fn register_external(&mut self, socket: ExternalSocket) -> Token{
        self.index = self.index+1;
        let token = Token(self.index);
        match self.poll.register(&socket, token, Ready::readable(), PollOpt::level()){
            Ok(_) => (),
            Err(_) => (),
        };
        self.external_map.insert(token,socket);
        self.type_map.insert(token,1);
        token
    }
    pub fn register_netbal(&mut self, socket: NetbalSocket) -> Token{
        self.index = self.index+1;
        let token = Token(self.index);
        match self.poll.register(&socket, token, Ready::readable(), PollOpt::level()){
            Ok(_) => (),
            Err(_) => (),
        };
        self.netbal_map.insert(token,socket);
        self.type_map.insert(token,2);
        token
    }
    pub fn register_timer(&mut self, timer: Timer<String>) -> Token{
        self.index = self.index+1;
        let token = Token(self.index);
        match self.poll.register(&timer, token, Ready::readable(), PollOpt::edge()){
            Ok(_) => (),
            Err(_) => (),
        };
        self.timer_map.insert(token,timer);
        self.type_map.insert(token,3);
        token
    }
}
impl Iterator for Events {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        match self.events.iter().next() {
            Some(event) => Some(event.token()),
            None=> None
        }
    }
}