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
    sync::{Arc, RwLock},
    // collections::HashMap,
    thread::{Builder, JoinHandle},
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

    ///
    ///
    ///     String -> UdpPacket IO Functions
    ///
    ///
    ///

    pub fn dump_bytes(&mut self, idx: usize, length: usize, data: &[u8]) {
        self.buffer[idx + 1..length + idx + 1].copy_from_slice(data);
    }

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
            .map(|_| {
                let i = idx;
                idx += 6;
                self.parse_addr(i)
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

    pub fn parse_name_packet(&self) -> (String, Vec<String>) {
        (self.sender_name(), self.parse_strings(self.data_start()))
    }

    pub fn parse_addr_packet(&self) -> (String, Vec<SocketAddr>) {
        (self.sender_name(), self.parse_addrs(self.data_start()))
    }

    pub fn parse_data_packet(&self) -> (String, Vec<f64>) {
        (self.sender_name(), self.parse_floats(self.data_start()))
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

pub fn empty_cb(_: &mut Sockage, _: &Vec<String>) {}

pub struct Sockage {
    pub name: String,
    pub lifetime: Instant,
    pub attention: Instant,
    pub micros_rate: u128,
    pub send_count: u128,
    pub recv_count: u128,

    pub targets_configured: bool,

    pub data: Vec<Vec<f64>>,
    pub timestamp: Vec<f64>,

    pub socket: UdpSocket,

    pub shutdown: Arc<RwLock<bool>>,

    pub socks: Vec<Sock>,
}

impl Sockage {
    pub fn new(name: &str, addr: SocketAddr, rate: u128) -> Sockage {
        Sockage {
            name: name.to_string(),
            lifetime: Instant::now(),
            attention: Instant::now(),
            micros_rate: rate,
            send_count: 0,
            recv_count: 0,

            targets_configured: false,

            data: vec![],
            timestamp: vec![],

            socket: Sockage::new_socket(addr, 1),

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
        Sockage::new(name, sock_uri!(1313), 200)
    }

    pub fn client(name: &str) -> Sockage {
        Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200)
    }

    pub fn sender(name: &str) -> Sockage {
        let mut sock = Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200);
        sock.sender_connect();
        sock
    }

    pub fn receiver(name: &str, names: &Vec<String>) -> Sockage {
        let mut sock = Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200);
        sock.data = vec![vec![]; names.len()];
        sock.timestamp = vec![0.0; names.len()];
        sock.data_request(names);
        sock
    }

    pub fn full(name: &str, names: &Vec<String>) -> Sockage {
        let mut sock = Sockage::new(name, sock_uri!(0, 0, 0, 0, 0), 200);
        sock.sender_connect();
        sock.data = vec![vec![]; names.len()];
        sock.timestamp = vec![0.0; names.len()];
        sock.data_request(names);
        sock
    }

    pub fn is_shutdown(&self) -> bool {
        *self.shutdown.read().unwrap()
    }

    pub fn shutdown(&self, status: bool) {
        *self.shutdown.write().unwrap() = status;
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
            None => self.add_sock(name, sock_uri!(0, 0, 0, 0, 0), vec![target.clone()]),
        });
    }

    pub fn register_names(&mut self, names: &Vec<String>) {
        names.iter().for_each(|name| {
            match self.find_name(name) {
                Some(_) => {}
                None => {
                    self.targets_configured = false;
                    self.add_sock(name, sock_uri!(0, 0, 0, 0, 0), vec![])
                }
            };
        });
    }

    pub fn register_addrs(&mut self, addrs: &Vec<SocketAddr>) {
        match addrs.len() == self.socks.len() {
            true => {
                self.targets_configured = true;
                addrs.iter().enumerate().for_each(|(i, addr)| {
                    if addr.port() > 0 {
                        self.socks[i].address = *addr;
                    } else {
                        self.targets_configured = false;
                    }
                });
            }

            false => self.targets_configured = false,
        };
    }

    pub fn discover_sock(&mut self, name: &str, addr: &SocketAddr) {
        match self.find_name(&name) {
            Some(i) => self.socks[i].address = *addr,
            _ => self.add_sock(name, *addr, vec![]),
        };
    }

    pub fn send_target_updates(&mut self, names: &Vec<String>) {
        names.iter().for_each(|name| {
            // self.log(format!("Sending names {:?}", self.targets(name)));
            // self.log(format!("Sending addrs {:?}", self.address_of_names(&self.targets(name))));
            let addr = self.address_of_name(name);
            match addr.port() > 0 {
                true => {
                    self.send_to(
                        SockBuffer::names_packet(&self.name, &self.targets(name)),
                        self.address_of_name(name),
                    );
                    self.send_to(
                        SockBuffer::addrs_packet(
                            &self.name,
                            &self.address_of_names(&self.targets(name)),
                        ),
                        self.address_of_name(name),
                    );
                }
                false => {}
            }
        });
    }

    pub fn send_terminate(&mut self) {
        let mut buffer = SockBuffer::stamp_packet(&self.name);
        buffer[0] = 13;
        (0..self.socks.len()).for_each(|i| {
            self.send_to(buffer, self.socks[i].address);
        });
    }

    pub fn data_request(&mut self, names: &Vec<String>) {
        let names_packet = SockBuffer::names_packet(&self.name, names);
        self.send_to(names_packet, sock_uri!(1313));
    }

    pub fn data_broadcast(&mut self, data: &Vec<f64>) {
        let data_packet = SockBuffer::data_packet(&self.name, data);

        (0..self.socks.len()).for_each(|i| {
            match self.socks[i].address.port() > 0 {
                true => {
                    // self.log(self.socks[i].name.clone());
                    self.send_to(data_packet, self.socks[i].address);
                }
                _ => {}
            };
        });
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

    pub fn peek(&mut self) -> SocketAddr {
        match !self.is_shutdown() {
            true => {
                let mut buffer = [0; 10];
                match self.socket.peek_from(&mut buffer) {
                    Ok((_size, src_addr)) => src_addr,

                    Err(_) => sock_uri!(0, 0, 0, 0, 0),
                }
            }
            false => sock_uri!(0, 0, 0, 0, 0),
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

    pub fn core_parse(&mut self) {
        let mut buffer = SockBuffer::new();
        let src = self.recv(&mut buffer.buffer);

        match (buffer.mode(), src.port() > 0) {
            (0, true) => {
                // self.send_to(SockBuffer::stamp_packet(&self.name), src);
                // self.log(format!("Stamp Request {:?}", src));
            }
            (1, true) => {
                let (sender, names) = buffer.parse_name_packet();
                self.discover_sock(&sender, &src);

                // self.log(format!("New Receiver {}", sender));

                self.add_target_to_names(&names, sender);
                // self.send_target_updates(&names); // send data to nodes registered as sender
            }
            (2, true) => {
                self.log(format!("Received 2 from {:?}", src));
            }
            (3, true) => {
                self.log(format!("Received 3 from {:?}", src));
            }
            (4, true) => {
                let sender = buffer.sender_name();

                // self.log(format!("New Sender: {}", sender));

                self.discover_sock(&sender, &src);
                self.send_target_updates(&vec![sender]); // send data to nodes registered as sender
            }
            (13, true) => {
                self.shutdown(true);
                self.send_terminate();
            }
            _ => {}
        };

        // if self.attention.elapsed().as_secs() > 0 {
        //     //     let names = (0..self.socks.len())
        //     //         .filter(|i| self.socks[*i].targets.len() > 0)
        //     //         .map(|i| self.socks[i as usize].name.clone())
        //     //         .collect();
        //     //     self.send_target_updates(&names);
        //     self.attention = Instant::now();
        //     self.log_heavy();
        // }
    }

    pub fn client_parse<F: Fn(&mut Sockage, &Vec<String>)>(
        &mut self,
        names: &Vec<String>,
        callback: F,
    ) {
        let mut buffer = SockBuffer::new();
        let src = self.recv(&mut buffer.buffer);

        match (src.port(), buffer.mode()) {
            (0, _) => {}
            (1313, 13) => self.shutdown(true),
            (1313, 1) => {
                let (_, names) = buffer.parse_name_packet();
                self.register_names(&names);
                // self.log(format!("Names {names:?}"));
            }
            (1313, 2) => {
                let (_, addrs) = buffer.parse_addr_packet();
                self.register_addrs(&addrs);
            }
            (_, 4) => {
                let (sender, data) = buffer.parse_data_packet();
                match names.iter().position(|name| *name == sender) {
                    Some(idx) => {
                        self.data[idx] = data;
                        self.timestamp[idx] = (self.lifetime.elapsed().as_micros() as f64) * 1E-3;

                        // match (0..self.timestamp.len()).find(|i| {
                        //     (self.lifetime.elapsed().as_millis() as f64 - self.timestamp[*i]) as f64
                        //         >= self.micros_rate as f64 / 200.0
                        // }) {
                        //     None => {
                        // self.timestamp = vec![(self.lifetime.elapsed().as_micros() as f64) * 1E-3]
                        callback(self, &names);
                        // }
                        // Some(_) => {}
                        // }
                    }

                    None => {}
                }
            }
            _ => {}
        }
    }

    pub fn sender_connect(&mut self) {
        self.send_to(
            SockBuffer::data_packet(&self.name, &vec![]),
            sock_uri!(1313),
        );

        self.targets_configured = false;

        let t = Instant::now();
        while t.elapsed().as_secs() < 1 && !self.targets_configured {
            self.client_parse(&vec![], empty_cb);
        }

        // match self.targets_configured {
        //     true => {
        //         // self.log_heavy();
        //         self.log(format!("Connected to core {}", t.elapsed().as_micros()));
        //     }
        //     false => {}
        // }
    }

    pub fn send(&mut self, data: Vec<f64>) {
        // let t = Instant::now();
        // if self.peek().port() == 1313 {
        //     self.client_parse(&vec![], empty_cb);
        //     println!("peek time {}", t.elapsed().as_micros());
        // }
        if self.attention.elapsed().as_secs() > 1 || !self.targets_configured {
            self.sender_connect();
            self.attention = Instant::now();
        }

        // match  {
        //     false => self.sender_connect(),
        //     _ => {}
        // };

        self.data_broadcast(&data);
    }

    pub fn receiver_spin<F: Fn(&mut Sockage, &Vec<String>)>(
        &mut self,
        names: Vec<String>,
        callback: F,
    ) {
        while !self.is_shutdown() {
            self.client_parse(&names, &callback);
        }
    }

    pub fn thread<F: Fn(&mut Sockage, &Vec<String>) + std::marker::Send + 'static>(
        name: String,
        names: Vec<String>,
        callback: F,
    ) -> JoinHandle<()> {
        let mut sock = Sockage::full(&name, &names);

        Builder::new()
            .name(name)
            .spawn(move || {
                sock.receiver_spin(names, callback);
            })
            .unwrap()
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

    pub fn echo(names: Vec<String>) {
        let mut sock = Sockage::receiver("echo-sock", &names);

        pub fn echo_callback(sock: &mut Sockage, names: &Vec<String>) {
            names
                .iter()
                .zip(sock.data.iter())
                .for_each(|(name, data)| sock.log(format!("[{name}] {data:?}")))
        }

        sock.receiver_spin(names, &echo_callback);

        sock.log_heavy();
    }

    pub fn hz(names: Vec<String>) {
        let mut sock = Sockage::receiver("hz-sock", &names);

        pub fn hz_callback(sock: &mut Sockage, _: &Vec<String>) {
            sock.log(1E6 / (sock.attention.elapsed().as_micros() as f64));
            sock.attention = Instant::now();
        }

        sock.receiver_spin(names, &hz_callback);

        sock.log_heavy();
    }

    pub fn shutdown_socks() {
        let mut sock = Sockage::client("kill-sock");
        let mut buffer = SockBuffer::stamp_packet(&sock.name);

        buffer[0] = 13;
        sock.send_to(buffer, sock_uri!(1313));
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
