/********************************************************************************
 *
 *      ____                     ____          __           __       _
 *     / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
 *    / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
 *   / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  )
 *  /_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/
 *        /____/
 *
 *
 *
 ********************************************************************************/
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    // sync::{Arc, RwLock},
    // thread::{Builder, JoinHandle},
    time::{Duration, Instant},
};

use socket2::{Domain, Protocol, Socket, Type};

use crate::ipv4;
use crate::sock_uri;
use crate::socks::data_structures::*;

pub const SOCK_NAME_LEN_IDX:    usize = 0;
pub const SOCK_NUM_TXS_IDX:     usize = SOCK_NAME_LEN_IDX + 1;
pub const SOCK_NUM_RXS_IDX:     usize = SOCK_NUM_TXS_IDX + 8;
pub const SOCK_ACTIVITY_IDX:    usize = SOCK_NUM_RXS_IDX + 8;
pub const SOCK_NAME_IDX:        usize = SOCK_ACTIVITY_IDX + 8;
pub const MAX_SOCK_NAME_LEN:    usize = SOCK_HEADER_LEN - SOCK_NAME_IDX;

// pub fn empty_cb(_: &mut Sockage, _: &Vec<String>) {}

fn new_multicast() -> UdpSocket {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    socket.set_nonblocking(true).unwrap();
    // socket.set_reuse_address(true).unwrap();

    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    socket.bind(&DEFAULT_URI.try_into().unwrap()).unwrap();
    socket
        .join_multicast_v4(&MULTICAST_IP, &Ipv4Addr::UNSPECIFIED)
        .expect("could not join multicast");

    socket.try_into().unwrap()
}

pub struct Sock {
    pub socket: UdpSocket,
    pub lifetime: Instant,
    pub activity: Instant,
    pub ntx: i64,
    pub nrx: i64,

    pub name: String,

    pub targets: Vec<String>,
    pub messages: Vec<Message>,
}

impl Sock {
    pub fn new(name: &str, targets: Vec<&str>) -> Sock {
        if name.as_bytes().len() > MAX_SOCK_NAME_LEN {
            panic!("Node name {name} is to long");
        }

        Sock {
            socket: new_multicast(),
            lifetime: Instant::now(),
            activity: Instant::now(),
            ntx: 0,
            nrx: 0,

            name: name.to_string(),

            targets: targets.iter().map(|target| target.to_string()).collect(),
            messages: vec![Message::new(); targets.len()],
        }
    }

    pub fn header_from_bytes(buffer: &[u8]) -> (String, i64, i64, f64) {
        let name_len = buffer[SOCK_NAME_LEN_IDX] as usize;
        let ntx = i64::from_be_bytes(get8_bytes(SOCK_NUM_TXS_IDX, buffer));
        let nrx = i64::from_be_bytes(get8_bytes(SOCK_NUM_TXS_IDX, buffer));
        let activity = f64::from_be_bytes(get8_bytes(SOCK_ACTIVITY_IDX, buffer));
        let name =
            String::from_utf8(buffer[SOCK_NAME_IDX..name_len + SOCK_NAME_IDX].to_vec()).unwrap();

        (name, ntx, nrx, activity)
    }

    pub fn header_bytes(&self) -> [u8; SOCK_HEADER_LEN] {
        let pad = MAX_SOCK_NAME_LEN - self.name.as_bytes().len();
        [self.name.as_bytes().len() as u8]
            .into_iter()
            .chain(self.ntx.to_be_bytes())
            .chain(self.nrx.to_be_bytes())
            .chain((self.activity.elapsed().as_micros() as f64 * 1E-6).to_be_bytes())
            .chain(self.name.as_bytes().to_vec())
            .chain(vec![0; pad])
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap()
    }

    pub fn tx(&mut self, mut buffer: UdpPacket, addr: SocketAddr) -> bool {
        match self.socket.send_to(&mut buffer, addr) {
            Ok(_) => {
                self.ntx += 1;
                true
            }
            Err(_) => false,
        }
    }

    pub fn rx(
        &mut self,
        buffer: &mut UdpPacket,
    ) -> Option<([u8; SOCK_HEADER_LEN], MessageFragment)> {
        match self.socket.recv_from(buffer) {
            Ok((_, _)) => {
                self.nrx += 1;
                Some(MessageFragment::from_bytes(*buffer))
            }
            Err(_) => None,
        }
    }

    pub fn peek(&mut self) -> SocketAddr {
        let mut buffer = [0; 10];
        match self.socket.peek_from(&mut buffer) {
            Ok((_size, src_addr)) => src_addr,

            Err(_) => sock_uri!(0, 0, 0, 0, 0),
        }
    }

    pub fn is_target(&self, name: &String) -> Option<usize> {
        (0..self.targets.len()).find(|&i| self.targets[i as usize] == *name)
    }

    pub fn collect(
        &mut self,
        idx: usize,
        header: [u8; SOCK_HEADER_LEN],
        fragment: MessageFragment,
    ) -> Option<usize> {
        self.messages[idx].header = header;
        match self.messages[idx].collect(fragment) {
            true => Some(idx),
            _ => None,
        }
    }

    pub fn try_rx(&mut self, buffer: &mut UdpPacket) -> Option<usize> {
        match self.rx(buffer) {
            Some((header, fragment)) => {
                let (name, _, _, _) = Sock::header_from_bytes(&header);
                match self.is_target(&name) {
                    Some(i) => self.collect(i, header, fragment),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn log<T: std::fmt::Debug>(&self, message: T) {
        println!(
            "[{:?}]:{:?}-{}<{},{}>\t\t{:?}\t({}s)",
            self.name,
            self.socket.local_addr().unwrap(),
            self.is_shutdown(),
            self.send_count,
            self.recv_count,
            message,
            self.lifetime.elapsed().as_micros() as f64 * 1E-6,
        );
    }

    pub fn log_heavy(&self) {
        println!(
            "==[Sockage]==\n[{:?}]:{}-{}<{},{}>\t({}s)\nSocks:\n\tnames:\n {:?}\n\taddress:\n{:?}\n\ttargets:\n{:?}",
            self.name,
            self.socket.local_addr().unwrap(),
            self.is_shutdown(),
            self.send_count,
            self.recv_count,
            self.lifetime.elapsed().as_micros() as f64 * 1E-6,
            self.socks.iter().map(|s| s.name.clone()).collect::<Vec<String>>(),
            self.socks.iter().map(|s| s.address.clone()).collect::<Vec<SocketAddr>>(),
            self.socks.iter().map(|s| s.targets.clone()).collect::<Vec<Vec<String>>>(),
        );
    }
}

// pub struct Sockage {
//     pub lifetime: Instant,
//     pub micros_rate: u128,

//     pub socket: UdpSocket,
//     pub shutdown: Arc<RwLock<bool>>,

//     pub node: Sock,

//     pub messages: Vec<Message>,
// }

// impl Sockage {
//     pub fn new(name: &str, rate: u128) -> Sockage {
//         Sockage {
//             lifetime: Instant::now(),
//             micros_rate: rate,
//             server_packets: 0,

//             socket: new_multicast(),
//             shutdown: Arc::new(RwLock::new(false)),

//             node: Sock::new(name, rate, vec![]),

//             socks: vec![],
//         }
//     }

//     pub fn is_shutdown(&self) -> bool {
//         *self.shutdown.read().unwrap()
//     }

//     pub fn shutdown(&self, status: bool) {
//         *self.shutdown.write().unwrap() = status;
//     }

//     pub fn clean_registry(&mut self) {
//         self.socks.retain(|sock| sock.activity <= 5.0 / sock.rate);
//     }

//     pub fn find_name(&self, name: &str) -> Option<usize> {
//         self.socks.iter().position(|sock| sock.name == name)
//     }

//     pub fn find_address(&self, addr: SocketAddr) -> Option<usize> {
//         self.socks.iter().position(|sock| sock.address == addr)
//     }

//     pub fn targets(&self, name: &str) -> &Vec<String> {
//         match self.find_name(name) {
//             Some(i) => self.socks[i].targets,
//             None => &vec![],
//         }
//     }

//     pub fn address_of_name(&self, name: &str) -> SocketAddr {
//         match self.find_name(name) {
//             Some(i) => self.socks[i].address,
//             None => sock_uri!(0, 0, 0, 0, 0),
//         }
//     }

//     pub fn address_of_names(&self, names: &Vec<String>) -> Vec<SocketAddr> {
//         names
//             .iter()
//             .map(|name| self.address_of_name(name))
//             .collect()
//     }

//     pub fn add_target_to_names(&mut self, names: &Vec<String>, target: String) {
//         names.iter().for_each(|name| match self.find_name(&name) {
//             Some(i) => self.socks[i].add_target(target.clone()),
//             None => self.add_sock(name, sock_uri!(0, 0, 0, 0, 0), vec![target.clone()]),
//         });
//     }

//     pub fn register_names(&mut self, names: &Vec<String>) {
//         names.iter().for_each(|name| {
//             match self.find_name(name) {
//                 Some(_) => {}
//                 None => {
//                     self.targets_configured = false;
//                     self.add_sock(name, sock_uri!(0, 0, 0, 0, 0), vec![])
//                 }
//             };
//         });
//     }

//     pub fn register_addrs(&mut self, addrs: &Vec<SocketAddr>) {
//         match addrs.len() == self.socks.len() {
//             true => {
//                 self.targets_configured = true;
//                 addrs.iter().enumerate().for_each(|(i, addr)| {
//                     if addr.port() > 0 {
//                         self.socks[i].address = *addr;
//                     } else {
//                         self.targets_configured = false;
//                     }
//                 });
//             }

//             false => self.targets_configured = false,
//         };
//     }

//     pub fn discover_sock(&mut self, addr: &SocketAddr, buffer: &SockBuffer) -> u8 {
//         let (
//             name,
//             mode,
//             connect,
//             send_count,
//             recv_count,
//             rate,
//             timestamp,
//         ) = buffer.get_sock();

//         match (addr.port(), self.find_name(&name)) {
//             (0, _) => {}
//             (_, Some(i)) => {
//                 self.socks[i].send_count = send_count;
//                 self.socks[i].recv_count = recv_count;
//                 self.socks[i].rate = rate;
//                 self.socks[i].timestamp = timestamp;
//             }
//             _ => socks.push(sock),
//         };

//         buffer.buffer[0]
//     }

//     pub fn send_target_names(&mut self, name: &String) {
//         let addr = self.address_of_name(name);
//         match (addr.port(), self.find_name(name)) {
//             (0, _) => {},
//             (_, Some(i)) => self.send_to(self.socks[i].to_packet(1), addr),
//             _ => {},
//         };
//     }

//     pub fn send_target_addresses(&mut self, name: &String) {
//         let addr = self.address_of_name(name);
//         match (addr.port(), self.find_name(name)) {
//             (0, _) => {},
//             (_, Some(i)) => {
//                 let addrs = self.address_of_names(self.socks[i].targets);
//                 self.send_to(self.socks[i].to_addr_packet(addrs), addr);
//             }
//             _ => {},
//         };
//     }

//     pub fn send_terminate(&mut self) {
//         let mut buffer = self.node.to_packet(13);
//         (0..self.socks.len()).for_each(|i| {
//             self.send_to(buffer, self.socks[i].address);
//         });
//     }

//     pub fn data_request(&mut self, names: &Vec<String>) {
//         let packet = SockBuffer::target_packet(&self, names);
//         self.send_to(packet, sock_uri!(1313));
//         self.server_packets += 1;
//     }

//     pub fn data_broadcast(&mut self, data: &Vec<f64>) {
//         let data_packet = SockBuffer::data_packet(&self, data);

//         (0..self.socks.len()).for_each(|i| {
//             match self.socks[i].address.port() > 0 {
//                 true => {
//                     self.send_to(data_packet, self.socks[i].address);
//                 }
//                 _ => {}
//             };
//         });
//     }

//     pub fn send_to(&mut self, mut buffer: UdpPacket, addr: SocketAddr) -> bool {
//         match !self.is_shutdown() {
//             true => match self.socket.send_to(&mut buffer, addr) {
//                 Ok(_) => {
//                     self.send_count += 1;
//                     true
//                 }
//                 Err(_) => false,
//             },

//             false => false,
//         }
//     }

//     pub fn peek(&mut self) -> SocketAddr {
//         match !self.is_shutdown() {
//             true => {
//                 let mut buffer = [0; 10];
//                 match self.socket.peek_from(&mut buffer) {
//                     Ok((_size, src_addr)) => src_addr,

//                     Err(_) => sock_uri!(0, 0, 0, 0, 0),
//                 }
//             }
//             false => sock_uri!(0, 0, 0, 0, 0),
//         }
//     }

//     pub fn recv(&mut self, buffer: &mut UdpPacket) -> SocketAddr {
//         match !self.is_shutdown() {
//             true => match self.socket.recv_from(buffer) {
//                 Ok((_size, src_addr)) => {
//                     self.recv_count += 1;
//                     src_addr
//                 }

//                 Err(_) => sock_uri!(0, 0, 0, 0, 0),
//             },
//             false => sock_uri!(0, 0, 0, 0, 0),
//         }
//     }

//     pub fn is_syncronized(&self, curr_time: f64) -> bool {
//         match (0..self.timestamp.len())
//             .find(|&i| (curr_time - self.timestamp[i]) > 0.5 / self.rates[i])
//         {
//             None => true,
//             Some(_) => {
//                 // self.log(format!("{i} {} {}", self.timestamp[i], (curr_time - self.timestamp[i])));
//                 false
//             }
//         }
//     }

//     pub fn core_parse(&mut self) {
//         let mut buffer = SockBuffer::new();
//         let src = self.recv(&mut buffer.buffer);

//         match self.discover_sock(&src, &buffer) {
//             1 => {
//                 // request to be a target (add name to sender nodes)
//                 let (sender, names) = buffer.parse_name_packet();
//                 self.add_target_to_names(&names, sender);
//             }
//             2 => {
//                 // request for target names (socks can only request their own targets)
//                 self.send_target_names(&buffer.sender_name());
//             }
//             3 => {
//                 // request for target addrs (socks can only request their own targets)
//                 self.send_target_addresses(&buffer.sender_name());
//             }
//             4 => {
//                 // data packet, will fill with status info (node graph?, data flow graph?)
//                 let names = (0..self.socks.len())
//                     .map(|i| self.socks[i].name.clone())
//                     .collect();
//                 self.send_to(SockBuffer::names_packet(&self, &names), src);
//             }
//             5 => {
//                 // data packet, will fill with status info (node graph?, data flow graph?)
//                 let addrs = (0..self.socks.len())
//                     .map(|i| self.socks[i].address)
//                     .collect();
//                 self.send_to(SockBuffer::addrs_packet(&self, &addrs), src);
//             }
//             13 => {
//                 // I am the terminator... I'll be back
//                 self.shutdown(true);
//                 self.send_terminate();
//             }
//             _ => {}
//         };

//         // self.socks
//         //     .retain(|sock| sock.timestamp.elapsed().as_secs() < 2);
//     }

//     pub fn client_parse<F: Fn(&mut Sockage, &Vec<String>)>(
//         &mut self,
//         names: &Vec<String>,
//         callback: F,
//     ) {
//         let mut buffer = SockBuffer::new();
//         let src = self.recv(&mut buffer.buffer);

//         match (src.port(), buffer.mode()) {
//             (0, _) => {}
//             (1313, 13) => self.shutdown(true),
//             (1313, 2) => {
//                 let (_, names) = buffer.parse_name_packet();
//                 self.register_names(&names);
//                 // self.log(format!("{names:?}"));
//             }
//             (1313, 3) => {
//                 let (_, addrs) = buffer.parse_addr_packet();
//                 self.register_addrs(&addrs);
//                 // self.log(format!("{addrs:?}"));
//             }
//             (_, 4) => {
//                 let (sender, data) = buffer.parse_data_packet();
//                 match names.iter().position(|name| *name == sender) {
//                     Some(idx) => {
//                         // self.log(sender);

//                         self.data[idx] = data;
//                         self.rates[idx] = buffer.rate();
//                         self.timestamp[idx] = (self.lifetime.elapsed().as_micros() as f64) * 1E-6;

//                         match self.is_syncronized(self.timestamp[idx]) || self.data.len() == 1 {
//                             true => {
//                                 callback(self, &names);
//                                 self.timestamp = vec![0.0; self.data.len()];
//                             }
//                             _ => {}
//                         }
//                     }

//                     None => {}
//                 }
//             }
//             _ => {}
//         }
//     }

//     pub fn sender_connect(&mut self) {
//         self.targets_configured = false;

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 1 && !self.targets_configured {
//             let tt = Instant::now();

//             self.send_to(SockBuffer::names_packet(&self, &vec![]), sock_uri!(1313));
//             self.server_packets += 1;
//             self.client_parse(&vec![], empty_cb);

//             self.send_to(SockBuffer::addrs_packet(&self, &vec![]), sock_uri!(1313));
//             self.server_packets += 1;
//             self.client_parse(&vec![], empty_cb);

//             while tt.elapsed().as_micros() < self.micros_rate {}
//         }

//         // match self.targets_configured {
//         //     true => {
//         //         // self.log_heavy();
//         //         self.log(format!("Connected to core {}", t.elapsed().as_micros()));
//         //     }
//         //     false => {}
//         // }
//     }

//     pub fn send(&mut self, data: Vec<f64>) {
//         // let t = Instant::now();
//         // if self.peek().port() == 1313 {
//         //     self.client_parse(&vec![], empty_cb);
//         //     println!("peek time {}", t.elapsed().as_micros());
//         // }
//         if self.attention.elapsed().as_secs() > 0 || !self.targets_configured {
//             self.sender_connect();
//             self.attention = Instant::now();
//         }

//         // match  {
//         //     false => self.sender_connect(),
//         //     _ => {}
//         // };

//         self.data_broadcast(&data);
//     }

//     pub fn receiver_spin<F: Fn(&mut Sockage, &Vec<String>)>(
//         &mut self,
//         names: Vec<String>,
//         callback: F,
//     ) {
//         while !self.is_shutdown() {
//             self.client_parse(&names, &callback);
//         }
//     }

//     pub fn log<T: std::fmt::Debug>(&self, message: T) {
//         println!(
//             "[{:?}]:{:?}-{}<{},{}>\t\t{:?}\t({}s)",
//             self.name,
//             self.socket.local_addr().unwrap(),
//             self.is_shutdown(),
//             self.send_count,
//             self.recv_count,
//             message,
//             self.lifetime.elapsed().as_micros() as f64 * 1E-6,
//         );
//     }

//     pub fn log_heavy(&self) {
//         println!(
//             "==[Sockage]==\n[{:?}]:{}-{}<{},{}>\t({}s)\nSocks:\n\tnames:\n {:?}\n\taddress:\n{:?}\n\ttargets:\n{:?}",
//             self.name,
//             self.socket.local_addr().unwrap(),
//             self.is_shutdown(),
//             self.send_count,
//             self.recv_count,
//             self.lifetime.elapsed().as_micros() as f64 * 1E-6,
//             self.socks.iter().map(|s| s.name.clone()).collect::<Vec<String>>(),
//             self.socks.iter().map(|s| s.address.clone()).collect::<Vec<SocketAddr>>(),
//             self.socks.iter().map(|s| s.targets.clone()).collect::<Vec<Vec<String>>>(),
//         );
//     }
// }

// struct SockApi {};

// impl SockApi{
//     pub fn core(name: &str) -> Sockage {
//         Sockage::new(name, sock_uri!(1313), 200)
//     }

//     pub fn client(name: &str) -> Sockage {
//         Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200)
//     }

//     pub fn sender(name: &str) -> Sockage {
//         let mut sock = Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200);
//         sock.sender_connect();
//         sock
//     }

//     pub fn receiver(name: &str, names: &Vec<String>) -> Sockage {
//         let mut sock = Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200);
//         sock.data = vec![vec![]; names.len()];
//         sock.timestamp = vec![0.0; names.len()];
//         sock.rates = vec![0.0; names.len()];
//         sock.data_request(names);
//         sock
//     }

//     pub fn full(name: &str, names: &Vec<String>) -> Sockage {
//         let mut sock = Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200);
//         sock.sender_connect();
//         sock.data = vec![vec![]; names.len()];
//         sock.timestamp = vec![0.0; names.len()];
//         sock.rates = vec![0.0; names.len()];
//         sock.data_request(names);
//         sock
//     }

//     pub fn thread<F: Fn(&mut Sockage, &Vec<String>) + std::marker::Send + 'static>(
//         name: String,
//         names: Vec<String>,
//         callback: F,
//     ) -> JoinHandle<()> {
//         let mut sock = Sockage::full(&name, &names);

//         Builder::new()
//             .name(name)
//             .spawn(move || {
//                 sock.receiver_spin(names, callback);
//                 sock.log_heavy();
//             })
//             .unwrap()
//     }

//     pub fn echo(names: Vec<String>) {
//         let mut sock = Sockage::receiver("echo-sock", &names);

//         pub fn echo_callback(sock: &mut Sockage, names: &Vec<String>) {
//             names
//                 .iter()
//                 .zip(sock.data.iter())
//                 .for_each(|(name, data)| sock.log(format!("[{name}] {data:?}")))
//         }

//         sock.receiver_spin(names, &echo_callback);

//         sock.log_heavy();
//     }

//     pub fn hz(names: Vec<String>) {
//         let mut sock = Sockage::receiver("hz-sock", &names);

//         pub fn hz_callback(sock: &mut Sockage, _: &Vec<String>) {
//             sock.log(1E6 / (sock.attention.elapsed().as_micros() as f64));
//             sock.attention = Instant::now();
//         }

//         sock.receiver_spin(names, &hz_callback);

//         sock.log_heavy();
//     }

//     pub fn shutdown_socks() {
//         let mut sock = Sockage::client("kill-sock");
//         let mut buffer = SockBuffer::stamp_packet(&sock);

//         buffer[0] = 13;
//         sock.send_to(buffer, sock_uri!(1313));
//     }
// }
