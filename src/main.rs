// mod config_parser;
use netbal::event::{EventType,EventManager};

use netbal::socket::{ExternalSocket,NetbalSocket};
use netbal::interface::Interface;
use netbal::client::Client;

use std::error::Error;
use mio::Token;
use std::path::PathBuf;
mod config_parser;
use nix::sys::stat::Mode;
use nix::sched::setns;
use nix::fcntl::open;
use std::time::{SystemTime,Duration};
use nix::unistd::{getpid, gettid};
use clap::{Arg, App};
use log;
use indexmap::IndexMap;
use mio_extras::timer::Timer;


fn main() -> Result<(),Box<dyn Error>> {

    let args = App::new("Netbal")
        .version("0.0.1")
        .author("Max Campbell <nb@0m.ax>")
        .about("Balance net load over multiple interfaces")
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .takes_value(true)
                 .help("Config file"))
        .get_matches();
    env_logger::init();

    let sys_time = SystemTime::now();

    //get applications current and default network namespace before it is changed
    let default_netns_fd = open(&PathBuf::from(format!("/proc/{}/task/{}/ns/net", getpid(), gettid())), nix::fcntl::OFlag::O_RDONLY, Mode::empty())?;

    let mut ev_manager = EventManager::new()?;

    let mut timer:Timer<String> = Timer::default();
    timer.set_timeout(Duration::from_millis(1000), "update".to_string());
    ev_manager.register_timer(timer);

    //load config
    let config_file = args.value_of("config").unwrap_or("./config.json").to_string();
    log::debug!("config_file {}",config_file);
    let c = config_parser::parse_config(config_file)?;
    //setup sockets
    let mut netbal_socket_tokens:IndexMap<usize,Token> = IndexMap::new();
    for socket_config in c.sockets {
        let ns_fd = match socket_config.netns {
            Some(netns)=>open(&PathBuf::from(netns), nix::fcntl::OFlag::O_RDONLY, Mode::empty())?,
            None => default_netns_fd
        };
        setns(ns_fd, nix::sched::CloneFlags::CLONE_NEWNET)?;
        let token = ev_manager.register_netbal(NetbalSocket::bind(socket_config.address.parse()?)?);
        netbal_socket_tokens.insert(socket_config.id, token);
    }

    //setup clients
    let mut clients:IndexMap<u16,Client> = IndexMap::new();
    let mut extern_socket_token_to_client_id:IndexMap<Token,u16> = IndexMap::new();
    for client_config in c.clients {
        //setup client interfaces
        let mut interfaces:IndexMap<u8,Interface> = IndexMap::new();
        for interface_config in client_config.interfaces {
            match netbal_socket_tokens.get(&interface_config.socket_id){
                Some(token)=>{
                    let interface = Interface::new(interface_config.weight,interface_config.id,interface_config.upstream.parse()?,interface_config.name,*token,true,client_config.id);
                    interfaces.insert(interface_config.id, interface);
                }
                _ =>{  unreachable!("could not find socket id {}",interface_config.socket_id); }
            };
        };
        // change to sockets network namespace
        let ns_fd = match client_config.external_socket.netns {
            Some(netns)=>open(&PathBuf::from(netns), nix::fcntl::OFlag::O_RDONLY, Mode::empty())?,
            None => default_netns_fd
        };
        setns(ns_fd, nix::sched::CloneFlags::CLONE_NEWNET)?;
        // create and register socket
        let token = ev_manager.register_external(ExternalSocket::bind(client_config.external_socket.address.parse()?)?);
        
        let client = Client {
            interfaces:interfaces,
            external_socket:token,
            id:client_config.id,
            external_upstream:client_config.external_socket.upstream.parse()?,
            allow_discover:client_config.allow_discover,
            next_interface_index:0
        };
        extern_socket_token_to_client_id.insert(token, client.id);
        clients.insert(client_config.id, client);
    };
    //main loop
    loop {
        let events = ev_manager.poll();

        for event in events.iter() {
            let token = event.token();
            match ev_manager.get(event.token()) {

                //inter applcation packets
                EventType::NetbalSocket(rx_socket) => {
                    let (packet,src) = rx_socket.recv_from()?;
                    match clients.get_mut(&packet.cid) {
                        Some(client) => {
                            let interface = match client.interfaces.get_mut(&packet.iid) {
                                Some(interface) => Some(interface),
                                _ => {
                                    if client.allow_discover {
                                        client.interfaces.insert(packet.iid, Interface::new(1,packet.iid,src,"auto find".to_string(),token,false,client.id));
                                        client.interfaces.get_mut(&packet.iid)
                                    }else{
                                        None
                                    }   
                                }
                            };
                            match interface {
                                Some(interface) => {
                                    interface.last_rx = sys_time.elapsed()?;
                                    if packet.data_packet {
                                        match ev_manager.get(client.external_socket) {
                                            EventType::ExternalSocket(tx_socket) => {
                                                interface.last_tx = sys_time.elapsed()?;
                                                tx_socket.send_to(&packet.data, client.external_upstream)?;
                                            }
                                            _ => {
                                                unreachable!("could not find external socket")
                                            }
                                        }
                                    }
                                },
                                _ => {
                                    println!("packet for unkown interface {}",packet.cid);
                                }
                            }
                        },
                        _ => {
                            println!("packet for unkown client {}",packet.cid);
                        }
                    }
                },

                //handle exernal packets
                EventType::ExternalSocket(rx_socket) => {
                    let (buf, src) = rx_socket.recv_from()?;
                    match extern_socket_token_to_client_id.get(&token){
                        Some(client_id) => {
                            match clients.get_mut(client_id) {
                                Some(client) => {
                                    client.next_interface_index = client.next_interface_index+1;
                                    if client.next_interface_index >= client.interfaces.len() {
                                        client.next_interface_index=0
                                    }
                                    match client.interfaces.get_index_mut(client.next_interface_index) {
                                        Some((_,interface)) => {
                                            match ev_manager.get(interface.socket_token) {
                                                EventType::NetbalSocket(tx_socket) => {
                                                    let packet = interface.make_packet(true, buf);
                                                    tx_socket.send_to(packet, interface.upstream)?;
                                                },
                                                _ => {
                                                    unreachable!("could not find socket")
                                                }
                                            };
                                            client.external_upstream =  src;
                                        }
                                        _ => {
                                             println!("could not find interface");
                                        }
                                    }
                                }
                                _ => unreachable!("could not find client for client id")
                            }
                        }
                        _ => unreachable!("could not find client id for external socket")
                    }
                }
                // Timer to handle keepaline packets

                EventType::Timer(timer) =>{
                    timer.set_timeout(Duration::from_millis(1000), "update".to_string());
                    for (_,client) in clients.iter_mut(){
                        let mut to_remove:Vec<u8> = Vec::new();
                        for (interface_id,interface) in client.interfaces.iter_mut() {
                            if !interface.is_static && interface.last_rx + Duration::from_millis(1000*30) < sys_time.elapsed()? {
                                to_remove.push(*interface_id);
                            }
                            if interface.last_tx + Duration::from_millis(1000*10) < sys_time.elapsed()? {
                                match ev_manager.get(interface.socket_token) {
                                    EventType::NetbalSocket(tx_socket) => {
                                        let packet = interface.make_packet(false, Vec::new());
                                        tx_socket.send_to(packet, interface.upstream)?;
                                        interface.last_tx = sys_time.elapsed()?;
                                    },
                                    _ => {
                                    }
                                };
                            }
                        }
                        for interface_id in to_remove{
                            client.interfaces.remove(&interface_id);
                        }
                    }
                }
                _ => unreachable!("unkown socket type")
            }
        };
    }
}
