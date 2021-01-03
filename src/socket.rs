use mio::net::UdpSocket;
use std::net::SocketAddr;
use crate::packet::OuterPacket;
use crate::processable::Processable;
use std::error::Error;
use mio::{Ready, Poll, PollOpt, Token};
use mio::event::Evented;
use std::io;

pub struct NetbalSocket {
    pub socket:UdpSocket,
}

impl NetbalSocket {
    pub fn bind(a:SocketAddr)->Result<Self, Box<dyn Error>> {
        return Ok(NetbalSocket {
            socket:UdpSocket::bind(&a)?
        });
    }
    pub fn recv_from(&self) ->  Result<(OuterPacket,SocketAddr),Box<dyn Error>> {
        let mut rev_buf:[u8; 65527] = [0; 65527];
        let (amt, src) = self.socket.recv_from(&mut rev_buf)?;
        return Ok((OuterPacket::decode(&rev_buf[..amt])?,src));
    }
    pub fn send_to(&self,p:OuterPacket,a:SocketAddr) ->  Result<(),Box<dyn Error>> {
        let _amt = self.socket.send_to(&p.encode()?,&a)?;
        return Ok(());
    }
}

impl Evented for NetbalSocket {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.socket.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.socket.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.socket.deregister(poll)
    }
}


pub struct ExternalSocket {
    pub socket:UdpSocket,
}

impl ExternalSocket {
    pub fn bind(a:SocketAddr)->Result<Self, Box<dyn Error>> {
        return Ok(ExternalSocket {
            socket:UdpSocket::bind(&a)?
        });
    }
    pub fn recv_from(&self) ->  Result<(Vec<u8>,SocketAddr),Box<dyn Error>> {
        let mut rev_buf:[u8; 65527] = [0; 65527];
        let (amt, src) = self.socket.recv_from(&mut rev_buf)?;
        return Ok((rev_buf[..amt].to_vec(),src));
    }
    pub fn send_to(&self,p:&[u8],a:SocketAddr) ->  Result<(),Box<dyn Error>> {
        let _amt = self.socket.send_to(&p,&a)?;
        return Ok(());
    }
}

impl Evented for ExternalSocket {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.socket.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
        -> io::Result<()>
    {
        self.socket.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.socket.deregister(poll)
    }
}


// poll.poll(&mut events, None)?;
// for event in events.iter() {
//     match event.token() {
//         EXTERNAL => {
//             let (amt, src) = extern_socket.recv_from(&mut rev_buf)?;
//             println!("ex rx {}",src);
//             last_extern_rcv = src;
//             match interfaces.choose_mut(&mut rng){
//                 Some(interface) =>{
//                     let p = packet::OuterPacket {
//                         stoc: true,
//                         pid: 0,
//                         cid: 0,
//                         iid: interface.iid,
//                         data:rev_buf[..amt].to_vec(),
//                     };
//                     println!("nb tx interface:{}",interface.name);
//                     interface.socket.send_to(p,interface.upstream)?;
//                 },
//                 None => {}
//             }
//         },
//         token => {
//             match token_to_index.get(&token) {
//                 Some(index) => {
//                     match interfaces.get_mut(*index) {
//                         Some(interface) => {
//                             println!("nb rx interface:{}",interface.name);
//                             let (p,_src) = interface.socket.recv_from()?;
//                             println!("ex tx {}",last_extern_rcv);
//                             extern_socket.send_to(&p.data,&last_extern_rcv)?;
//                             interface.last_rx = sys_time.elapsed()?;
//                         },
//                         None => {}
//                     }
//                 },
//                 None => {}
//             };
//         }
//     }
// }