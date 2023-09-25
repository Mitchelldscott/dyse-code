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
    sync::{
        // mpsc,
        // mpsc::{Receiver, Sender},
        Arc,
        RwLock,
    },
    // collections::HashMap,
    // thread::{Builder, JoinHandle},
    time::{Duration, Instant},
};

pub const UDP_PACKET_SIZE: usize = 1024;
type UdpPacket = [u8; UDP_PACKET_SIZE];

macro_rules! sock_uri {
    () => {{
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)
    }};
    ($port:expr) => {{
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), $port)
    }};
    ($ip1:expr, $ip2:expr, $ip3:expr, $ip4:expr, $port:expr) => {{
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new($ip1, $ip2, $ip3, $ip4)), $port)
    }};
}

pub struct SockBuffer {
    pub buffer: UdpPacket,
}

impl SockBuffer {
    pub fn new() -> SockBuffer {
        SockBuffer {
            buffer: [0u8; UDP_PACKET_SIZE],
        }
    }

    ///
    ///
    ///     helpers UdpPacket IO Functions
    ///
    ///
    ///

    pub fn buffer(&self) -> UdpPacket {
        self.buffer
    }

    pub fn mode(&self) -> u8 {
        self.buffer[0] // buffer[0..buffer[0]+1] = (name_len, mode, name)
    }

    pub fn set_mode(&mut self, value: u8) {
        self.buffer[0] = value; // buffer[0..1] = (name_len, mode)
    }

    pub fn name_len(&self) -> u8 {
        self.buffer[1] // buffer[0..buffer[0]+1] = (name_len, mode, name)
    }

    pub fn set_name_len(&mut self, value: u8) {
        self.buffer[1] = value; // buffer[0..1] = (name_len, mode)
    }

    pub fn data_start(&self) -> usize {
        (self.name_len() + 2) as usize // buffer[0]+2 = name_len+2
    }

    pub fn data_len(&self) -> usize {
        self.buffer[self.data_start()] as usize // data_len is always first byte in data section
    }

    pub fn dump_sender(&mut self, name: &str) {
        self.dump_string(1, name);
    }

    pub fn sender_name(&self) -> String {
        self.parse_string(1)
    }

    pub fn parse_name_packet(&self) -> (String, Vec<String>) {
        (self.sender_name(), self.parse_strings(self.data_start()))
    }

    pub fn parse_addr_packet(&self) -> (String, Vec<SocketAddr>) {
        (self.sender_name(), self.parse_addrs(self.data_start()))
    }

    pub fn parse_data_packet(&self) -> (String, Vec<f64>) {
        (self.sender_name(), self.parse_floats(self.data_start()))
    }

    ///
    ///
    ///     String -> UdpPacket IO Functions
    ///
    ///
    ///

    pub fn dump_string(&mut self, idx: usize, data: &str) -> usize {
        let data = data.as_bytes();
        let data_len = data.len();
        self.buffer[idx] = data_len as u8;
        self.buffer[idx + 1..data_len + idx + 1].copy_from_slice(&data);
        data_len + idx + 1
    }

    pub fn parse_string(&self, idx: usize) -> String {
        let str_len = self.buffer[idx] as usize;
        match String::from_utf8(self.buffer[idx + 1..str_len + idx + 1].to_vec()) {
            Ok(s) => s,
            Err(_) => String::new(),
        }
    }

    pub fn dump_strings(&mut self, mut idx: usize, names: &Vec<String>) {
        self.buffer[idx] = names.len() as u8;
        idx += 1;
        names.iter().for_each(|name| {
            idx = self.dump_string(idx, name);
        })
    }

    pub fn parse_strings(&self, mut idx: usize) -> Vec<String> {
        let n_data = self.buffer[idx] as usize;
        idx += 1;

        (0..n_data)
            .filter_map(|_| match idx < UDP_PACKET_SIZE {
                true => {
                    let name = self.parse_string(idx);
                    idx += (self.buffer[idx] + 1) as usize;
                    match name.len() > 0 {
                        true => Some(name),
                        false => None,
                    }
                }
                false => None,
            })
            .collect()
    }

    ///
    ///
    ///     f64 -> UdpPacket IO Functions
    ///
    ///
    ///

    pub fn dump_float(&mut self, idx: usize, data: f64) -> usize {
        let data = data.to_be_bytes();
        self.buffer[idx..idx + std::mem::size_of::<f64>()].copy_from_slice(&data);
        idx + std::mem::size_of::<f64>()
    }

    pub fn parse_float(&self, idx: usize) -> f64 {
        // gross but whatever ig
        f64::from_be_bytes([
            self.buffer[idx],
            self.buffer[idx + 1],
            self.buffer[idx + 2],
            self.buffer[idx + 3],
            self.buffer[idx + 4],
            self.buffer[idx + 5],
            self.buffer[idx + 6],
            self.buffer[idx + 7],
        ])
    }

    pub fn dump_floats(&mut self, mut idx: usize, data: &Vec<f64>) {
        self.buffer[idx] = data.len() as u8;
        idx += 1;
        data.iter().for_each(|value| {
            idx = self.dump_float(idx, *value);
        })
    }

    pub fn parse_floats(&self, mut idx: usize) -> Vec<f64> {
        let n_data = self.buffer[idx] as usize;
        idx += 1;

        (0..n_data)
            .filter_map(|_| match idx < UDP_PACKET_SIZE {
                true => {
                    let float = self.parse_float(idx);
                    idx += std::mem::size_of::<f64>();
                    Some(float)
                }
                false => None,
            })
            .collect()
    }

    ///
    ///
    ///     SocketAddr -> UdpPacket IO Functions
    ///
    ///
    ///

    pub fn dump_addr(&mut self, idx: usize, addr: &SocketAddr) -> usize {
        let mut octets = match addr.ip() {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            _ => {
                vec![0, 0, 0, 0]
            }
        };
        octets.append(&mut addr.port().to_be_bytes().to_vec());
        self.buffer[idx..idx + 6].copy_from_slice(&octets);
        idx + 6
    }

    pub fn parse_addr(&self, idx: usize) -> SocketAddr {
        sock_uri!(
            self.buffer[idx],
            self.buffer[idx + 1],
            self.buffer[idx + 2],
            self.buffer[idx + 3],
            u16::from_be_bytes([self.buffer[idx + 4], self.buffer[idx + 5]])
        )
    }

    pub fn dump_addrs(&mut self, mut idx: usize, ips: &Vec<SocketAddr>) {
        self.buffer[idx] = ips.len() as u8;
        idx += 1;
        ips.iter().for_each(|ip| {
            idx = self.dump_addr(idx, ip);
        });
    }

    pub fn parse_addrs(&self, mut idx: usize) -> Vec<SocketAddr> {
        let n_addrs = self.buffer[idx] as usize;
        idx += 1;

        (0..n_addrs)
            .filter_map(|_| match idx < UDP_PACKET_SIZE {
                true => {
                    let addr = self.parse_addr(idx);
                    idx += (self.buffer[idx] + 1) as usize;
                    Some(addr)
                }
                false => None,
            })
            .collect()
    }

    ///
    ///
    ///     UdpPacket Builder Functions
    ///
    ///
    ///

    pub fn stamp_packet(name: &str) -> UdpPacket {
        let mut packet = SockBuffer::new();
        packet.set_mode(0);
        packet.dump_sender(name);
        packet.buffer
    }

    pub fn names_packet(name: &str, names: &Vec<String>) -> UdpPacket {
        let mut packet = SockBuffer::new();
        packet.set_mode(1);
        packet.dump_sender(name);
        packet.dump_strings(packet.data_start(), names);
        packet.buffer
    }

    pub fn addrs_packet(name: &str, addrs: &Vec<SocketAddr>) -> UdpPacket {
        let mut packet = SockBuffer::new();
        packet.set_mode(2);
        packet.dump_sender(name);
        packet.dump_addrs(packet.data_start(), addrs);
        packet.buffer
    }

    pub fn data_request_packet(name: &str) -> UdpPacket {
        let mut packet = SockBuffer::new();
        packet.set_mode(3);
        packet.dump_sender(name);
        packet.buffer
    }

    pub fn data_packet(name: &str, data: &Vec<f64>) -> UdpPacket {
        let mut packet = SockBuffer::new();
        packet.set_mode(4);
        packet.dump_sender(name);
        packet.dump_floats(packet.data_start(), data);
        packet.buffer
    }
}

pub struct Sock {
    pub name: String,
    pub address: SocketAddr,
    pub targets: Vec<String>,
}

impl Sock {
    pub fn new(name: &str, address: SocketAddr, targets: Vec<String>) -> Sock {
        Sock {
            name: name.to_string(),
            address: address,
            targets: targets,
        }
    }

    pub fn add_target(&mut self, target: String) {
        match self.targets.iter().position(|t| *t == target) {
            None => self.targets.push(target),
            Some(_) => {}
        };
    }
}

pub struct Sockage {
    pub name: String,
    pub lifetime: Instant,
    pub millis_rate: u128,
    pub send_count: u128,
    pub recv_count: u128,

    pub socket: UdpSocket,

    pub shutdown: Arc<RwLock<bool>>,

    pub socks: Vec<Sock>,
}

impl Sockage {
    pub fn new(name: &str, addr: SocketAddr, rate: u128) -> Sockage {
        Sockage {
            name: name.to_string(),
            lifetime: Instant::now(),
            millis_rate: rate,
            send_count: 0,
            recv_count: 0,

            socket: Sockage::new_socket(addr, rate as u32),

            shutdown: Arc::new(RwLock::new(false)),

            socks: vec![],
        }
    }

    pub fn new_socket(addr: SocketAddr, timeout: u32) -> UdpSocket {
        let socket = UdpSocket::bind(addr).expect("Couldn't bind to socket");
        socket
            .set_write_timeout(Some(Duration::new(0, timeout)))
            .unwrap();
        socket
            .set_read_timeout(Some(Duration::new(0, timeout)))
            .unwrap();
        socket
    }

    pub fn core(name: &str) -> Sockage {
        Sockage::new(name, sock_uri!(1313), 1)
    }

    pub fn client(name: &str) -> Sockage {
        Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 1)
    }

    pub fn clear_registry(&mut self) {
        self.socks.clear();
    }

    pub fn find_name(&self, name: &str) -> Option<usize> {
        self.socks.iter().position(|sock| sock.name == name)
    }

    pub fn find_address(&self, addr: SocketAddr) -> Option<usize> {
        self.socks.iter().position(|sock| sock.address == addr)
    }

    pub fn targets(&self, target: &str) -> Vec<String> {
        match self.find_name(target) {
            Some(i) => self.socks[i].targets.clone(),
            None => vec![],
        }
    }

    pub fn address_of_name(&self, name: &str) -> SocketAddr {
        match self.find_name(name) {
            Some(i) => self.socks[i].address,
            None => sock_uri!(0, 0, 0, 0, 0),
        }
    }

    pub fn address_of_names(&self, names: &Vec<String>) -> Vec<SocketAddr> {
        names
            .iter()
            .map(|name| self.address_of_name(name))
            .collect()
    }

    pub fn add_sock(&mut self, name: &str, addr: SocketAddr, targets: Vec<String>) {
        match self.find_name(name) {
            Some(_) => {}
            _ => self.socks.push(Sock::new(name, addr, targets)),
        };
    }

    pub fn add_target_to_names(&mut self, names: &Vec<String>, target: String) {
        names.iter().for_each(|name| match self.find_name(&name) {
            Some(i) => self.socks[i].add_target(target.clone()),
            None => {}
        });
    }

    pub fn register_names(&mut self, names: &Vec<String>) {
        names.iter().for_each(|name| {
            match self.find_name(name) {
                Some(_) => {}
                None => self.add_sock(name, sock_uri!(0, 0, 0, 0, 0), vec![]),
            };
        });
    }

    pub fn register_addrs(&mut self, addrs: &Vec<SocketAddr>) {
        addrs.iter().enumerate().for_each(|(i, addr)| {
            self.socks[i].address = *addr;
        });
    }

    pub fn discover_sock(&mut self, name: &str, addr: &SocketAddr) {
        match self.find_name(&name) {
            Some(i) => self.socks[i].address = *addr,
            _ => self.add_sock(name, *addr, vec![]),
        }
    }

    pub fn send_target_updates(&mut self, names: &Vec<String>) {
        names.iter().for_each(|name| {
            self.send_to(
                SockBuffer::names_packet(&self.name, &self.targets(name)),
                self.address_of_name(name),
            );
            self.send_to(
                SockBuffer::addrs_packet(&self.name, &self.address_of_names(&self.targets(name))),
                self.address_of_name(name),
            );
        });
    }

    pub fn data_broadcast(&mut self, data: &Vec<f64>) {
        let data_packet = SockBuffer::data_packet(&self.name, data);
        (0..self.socks.len()).for_each(|i| {
            match self.socks[i].address.port() > 0 {
                true => {
                    self.send_to(data_packet, self.socks[i].address);
                }
                false => {}
            };
        })
    }

    pub fn send_to(&mut self, mut buffer: UdpPacket, addr: SocketAddr) -> bool {
        match !self.is_shutdown() {
            true => match self.socket.send_to(&mut buffer, addr) {
                Ok(_) => {
                    self.send_count += 1;
                    true
                }
                Err(_) => false,
            },

            false => false,
        }
    }

    pub fn recv(&mut self, buffer: &mut UdpPacket) -> SocketAddr {
        match !self.is_shutdown() {
            true => match self.socket.recv_from(buffer) {
                Ok((_size, src_addr)) => {
                    self.recv_count += 1;
                    src_addr
                }

                Err(_) => sock_uri!(0, 0, 0, 0, 0),
            },
            false => sock_uri!(0, 0, 0, 0, 0),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        *self.shutdown.read().unwrap()
    }

    pub fn shutdown(&self, status: bool) {
        *self.shutdown.write().unwrap() = status;
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

// pub struct FullDuplexChannel {
//     // Sopic or Sockic, idk
//     pub tx: Sender<UdpPacket>, // data from the server
//     pub rx: Receiver<UdpPacket>, // data from the user
// }

// impl FullDuplexChannel {
//     pub fn partner(
//         tx: Sender<UdpPacket>,
//         rx: Receiver<UdpPacket>,
//     ) -> FullDuplexChannel {
//         FullDuplexChannel { rx: rx, tx: tx }
//     }

//     pub fn new() -> (
//         FullDuplexChannel,
//         Sender<[u8; UDP_PACKET_SIZE]>,
//         Receiver<[u8; UDP_PACKET_SIZE]>,
//     ) {
//         let (partner_tx, rx): (
//             Sender<[u8; UDP_PACKET_SIZE]>,
//             Receiver<[u8; UDP_PACKET_SIZE]>,
//         ) = mpsc::channel();
//         let (tx, partner_rx): (
//             Sender<[u8; UDP_PACKET_SIZE]>,
//             Receiver<[u8; UDP_PACKET_SIZE]>,
//         ) = mpsc::channel();
//         (FullDuplexChannel::partner(tx, rx), partner_tx, partner_rx)
//     }

//     pub fn clone(&mut self) -> FullDuplexChannel {
//         let (partner_tx, rx): (
//             Sender<[u8; UDP_PACKET_SIZE]>,
//             Receiver<[u8; UDP_PACKET_SIZE]>,
//         ) = mpsc::channel();
//         let (tx, partner_rx): (
//             Sender<[u8; UDP_PACKET_SIZE]>,
//             Receiver<[u8; UDP_PACKET_SIZE]>,
//         ) = mpsc::channel();
//         self.tx = tx;
//         self.rx = rx;

//         FullDuplexChannel::partner(partner_tx, partner_rx)
//     }
// }
