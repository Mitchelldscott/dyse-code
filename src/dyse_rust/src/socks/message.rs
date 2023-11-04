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

use std::time::Instant;

pub const UDP_PACKET_SIZE: usize = 1024;
pub const SOCK_HEADER_LEN: usize = 64;
pub const FRAG_HEADER_LEN: usize = 10;
pub const PAYLOAD_IDX: usize = SOCK_HEADER_LEN + FRAG_HEADER_LEN;
pub const MAX_FRAGMENT_SIZE: usize = UDP_PACKET_SIZE - PAYLOAD_IDX;

pub type UdpPacket = [u8; UDP_PACKET_SIZE];
pub type UdpPayload = Vec<u8>;

pub fn get8_bytes(idx: usize, buffer: &[u8]) -> [u8; 8] {
    [
        buffer[idx],
        buffer[idx + 1],
        buffer[idx + 2],
        buffer[idx + 3],
        buffer[idx + 4],
        buffer[idx + 5],
        buffer[idx + 6],
        buffer[idx + 7],
    ]
}

#[derive(Clone)]
pub struct MessageFragment {
    pub offset: usize,
    pub n_bytes: usize,
    pub total_fragments: usize,
    pub payload: [u8; MAX_FRAGMENT_SIZE],
}

impl MessageFragment {
    pub fn default() -> MessageFragment {
        MessageFragment {
            offset: 0,
            n_bytes: 0,
            total_fragments: 0,
            payload: [0; MAX_FRAGMENT_SIZE],
        }
    }

    pub fn new(offset: usize, n_fragments: usize) -> MessageFragment {
        MessageFragment {
            offset: offset,
            n_bytes: 0,
            total_fragments: n_fragments,
            payload: [0; MAX_FRAGMENT_SIZE],
        }
    }

    pub fn from_shatter(
        offset: usize,
        n_fragments: usize,
        mut payload: Vec<u8>,
    ) -> MessageFragment {
        let payload_size = payload.len();

        if payload_size > MAX_FRAGMENT_SIZE {
            payload.truncate(MAX_FRAGMENT_SIZE);
        } else if payload_size < MAX_FRAGMENT_SIZE {
            payload.extend(vec![0; MAX_FRAGMENT_SIZE - payload_size]);
        }

        MessageFragment {
            offset: offset,
            n_bytes: payload_size,
            total_fragments: n_fragments,
            payload: payload.try_into().unwrap(),
        }
    }

    pub fn from_bytes(buffer: UdpPacket) -> ([u8; SOCK_HEADER_LEN], MessageFragment) {
        let n_bytes = usize::from_be_bytes(get8_bytes(SOCK_HEADER_LEN + 2, &buffer));
        (
            buffer[0..SOCK_HEADER_LEN].try_into().unwrap(),
            MessageFragment {
                offset: buffer[SOCK_HEADER_LEN] as usize,
                n_bytes: n_bytes,
                total_fragments: buffer[SOCK_HEADER_LEN + 1] as usize,
                payload: buffer[PAYLOAD_IDX..UDP_PACKET_SIZE].try_into().unwrap(),
            },
        )
    }

    pub fn to_bytes(&self, header: [u8; SOCK_HEADER_LEN]) -> UdpPacket {
        let mut buffer = [0; UDP_PACKET_SIZE];
        header
            .iter()
            .chain([self.offset as u8, self.total_fragments as u8].iter())
            .chain(self.n_bytes.to_be_bytes().iter())
            .chain(self.payload.iter())
            .enumerate()
            .for_each(|(i, b)| buffer[i] = *b);
        buffer
    }
}

#[derive(Clone)]
pub struct Message {
    pub fragments: Vec<MessageFragment>,
    pub timestamp: Instant,
    pub micros_rate: u64,
    pub ntx: i64,
}

impl Message {
    pub fn new() -> Message {
        Message {
            fragments: vec![],
            timestamp: Instant::now(),
            micros_rate: 0,
            ntx: 0,
        }
    }

    pub fn shatter(payload: Vec<u8>) -> Vec<MessageFragment> {
        // let shards: Vec<&[u8]> = match payload.len() > MAX_FRAGMENT_SIZE {
        //     true => payload.chunks(MAX_FRAGMENT_SIZE).collect(),
        //     false => vec![&payload],
        // };

        let shards: Vec<&[u8]> = payload.chunks(MAX_FRAGMENT_SIZE).collect();

        (0..shards.len())
            .map(|i| MessageFragment::from_shatter(i, shards.len(), shards[i].to_vec()))
            .collect()
    }

    pub fn from_payload(payload: Vec<u8>) -> Message {
        let fragments = Message::shatter(payload);

        Message {
            fragments: fragments,
            timestamp: Instant::now(),
            micros_rate: 0,
            ntx: 0,
        }
    }

    pub fn packets(&self, header: [u8; SOCK_HEADER_LEN]) -> Vec<UdpPacket> {
        (0..self.fragments.len())
            .map(|i| self.fragments[i].to_bytes(header))
            .collect()
    }

    pub fn init_fragments(&mut self, n: usize) {
        self.fragments = (0..n).map(|_| MessageFragment::new(255, n)).collect();
    }

    pub fn is_available(&self) -> bool {
        (self.micros_rate / 2) as u128 > self.timestamp.elapsed().as_micros()
    }

    pub fn collect(&mut self, ntx: i64, micros: u64, fragment: MessageFragment) -> bool {
        if self.fragments.len() == 0 || ntx > self.ntx || micros > 2 * self.micros_rate {
            self.init_fragments(fragment.total_fragments);
        }

        self.ntx = ntx;
        self.micros_rate = micros;
        self.fragments[fragment.offset] = fragment.clone();

        match (0..self.fragments.len()).find(|&i| self.fragments[i].offset != i) {
            Some(_) => false,
            None => {
                self.timestamp = Instant::now();
                true
            }
        }
    }

    pub fn to_payload(&self) -> UdpPayload {
        (0..self.fragments.len())
            .map(|i| self.fragments[i].payload[0..self.fragments[i].n_bytes].to_vec())
            .flatten()
            .collect()
    }
}

// pub struct SockBuffer {
//     pub buffer: UdpPacket,
// }

// impl SockBuffer {
//     ///
//     ///
//     ///     UdpPacket Structure
//     ///
//     /// [0:64] - header (sender info)
//     /// [64:UDP_PACKET_SIZE] - data
//     ///

//     pub fn new() -> SockBuffer {
//         SockBuffer {
//             buffer: [0u8; UDP_PACKET_SIZE],
//         }
//     }

//     ///
//     ///
//     ///     helpers UdpPacket IO Functions
//     ///
//     ///
//     ///

//     pub fn set_bytes(&mut self, idx: usize, length: usize, data: &[u8]) {
//         self.buffer[idx..length + idx].copy_from_slice(data);
//     }

//     pub fn get_bytes(&self, idx: usize, length: usize) -> &[u8] {
//         &self.buffer[idx..length + idx]
//     }

//     pub fn get8_bytes(&self, idx: usize) -> [u8; 8] {
//         [
//             self.buffer[idx],
//             self.buffer[idx + 1],
//             self.buffer[idx + 2],
//             self.buffer[idx + 3],
//             self.buffer[idx + 4],
//             self.buffer[idx + 5],
//             self.buffer[idx + 6],
//             self.buffer[idx + 7],
//         ]
//     }

//     pub fn set_sock(
//         &mut self,
//         name: &[u8],
//         mode: u8,
//         connect: u8,
//         send_count: i64,
//         recv_count: i64,
//         rate: f64,
//         timestamp: f64,
//     ) {
//         self.set_bytes(SOCK_MODE, 3, &[mode, connect, name.len() as u8]);
//         self.set_bytes(
//             SOCK_SEND_COUNT,
//             std::mem::size_of::<i64>(),
//             &send_count.to_be_bytes(),
//         );
//         self.set_bytes(
//             SOCK_RECV_COUNT,
//             std::mem::size_of::<i64>(),
//             &recv_count.to_be_bytes(),
//         );
//         self.set_bytes(SOCK_RATE, std::mem::size_of::<f64>(), &rate.to_be_bytes());
//         self.set_bytes(
//             SOCK_TIMESTAMP,
//             std::mem::size_of::<f64>(),
//             &timestamp.to_be_bytes(),
//         );
//         self.set_bytes(SOCK_NAME, name.len(), name);
//     }

//     pub fn get_sock(&self) -> (String, u8, u8, i64, i64, f64, f64) {
//         let mode = self.buffer[SOCK_MODE];
//         let connect = self.buffer[SOCK_CONNECTED];
//         let name_len = self.buffer[SOCK_NAME_LEN] as usize;
//         let send_count = i64::from_be_bytes(self.get8_bytes(SOCK_SEND_COUNT));
//         let recv_count = i64::from_be_bytes(self.get8_bytes(SOCK_RECV_COUNT));
//         let rate = f64::from_be_bytes(self.get8_bytes(SOCK_RATE));
//         let timestamp = f64::from_be_bytes(self.get8_bytes(SOCK_TIMESTAMP));
//         let name = String::from_utf8(self.get_bytes(SOCK_NAME, name_len).to_vec()).unwrap();

//         (name, mode, connect, send_count, recv_count, rate, timestamp)
//     }

//     pub fn data_start(&self) -> usize {
//         self.buffer[SOCK_NAME_LEN] as usize + SOCK_NAME
//     }

//     pub fn set_names(&mut self, names: &Vec<String>) {
//         let mut n_bytes = 0;
//         let bytes: Vec<u8> = names
//             .iter()
//             .map(|name| {
//                 let name_bytes = name.as_bytes().to_vec();
//                 n_bytes += name_bytes.len() + 1;
//                 [name_bytes.len() as u8].into_iter().chain(name_bytes)
//             })
//             .flatten()
//             .collect();

//         self.buffer[SOCK_DATA_LEN] = n_bytes as u8;
//         self.set_bytes(self.data_start(), n_bytes, bytes.as_slice());
//     }

//     pub fn get_names(&self) -> Vec<String> {
//         let mut targets = vec![];
//         let start = self.data_start();
//         let data_len = self.buffer[SOCK_DATA_LEN] as usize;

//         let mut i = start;
//         while i < start + data_len {
//             let name_len = self.buffer[i] as usize;
//             let name = match String::from_utf8(self.get_bytes(i + 1, name_len).to_vec()) {
//                 Ok(s) => s,
//                 Err(_) => String::new(),
//             };

//             targets.push(name);

//             i += name_len + 1;
//         }

//         targets
//     }

//     pub fn set_targets(&mut self, names: &Vec<String>, addrs: &Vec<SocketAddr>) {
//         let mut n_bytes = 0;
//         let bytes: Vec<u8> = names
//             .iter()
//             .zip(addrs.iter())
//             .map(|(name, addr)| {
//                 let name_bytes = name.as_bytes().to_vec();
//                 let addr_bytes = address_to_bytes(addr);
//                 n_bytes += name_bytes.len() + addr_bytes.len() + 1;
//                 [name_bytes.len() as u8]
//                     .into_iter()
//                     .chain(name_bytes.into_iter().chain(addr_bytes))
//             })
//             .flatten()
//             .collect();

//         self.buffer[SOCK_DATA_LEN] = n_bytes as u8;
//         self.set_bytes(self.data_start(), n_bytes, bytes.as_slice());
//     }

//     pub fn get_targets(&self) -> (Vec<String>, Vec<SocketAddr>) {
//         let mut addrs = vec![];
//         let mut targets = vec![];
//         let start = self.data_start();
//         let data_len = self.buffer[SOCK_DATA_LEN] as usize;

//         let mut i = start;
//         while i < start + data_len {
//             let name_len = self.buffer[i] as usize;
//             let name = match String::from_utf8(self.get_bytes(i + 1, name_len).to_vec()) {
//                 Ok(s) => s,
//                 Err(_) => String::new(),
//             };

//             let addr = address_from_bytes(self.get_bytes(name_len + i + 1, 6));

//             targets.push(name);
//             addrs.push(addr);

//             i += name_len + 7;
//         }

//         (targets, addrs)
//     }
// }

// // pub struct SockBuffer {
// //     pub buffer: UdpPacket,
// // }

// // impl SockBuffer {
// //     pub fn new() -> SockBuffer {
// //         SockBuffer {
// //             buffer: [0u8; UDP_PACKET_SIZE],
// //         }
// //     }

// //     ///
// //     ///
// //     ///     helpers UdpPacket IO Functions
// //     ///
// //     ///
// //     ///

// //     pub fn mode(&self) -> u8 {
// //         self.buffer[0] // buffer[0] = mode
// //     }

// //     pub fn set_mode(&mut self, value: u8) {
// //         self.buffer[0] = value;
// //     }

// //     pub fn name_len(&self) -> u8 {
// //         self.buffer[1] // buffer[1] = name_len
// //     }

// //     pub fn set_name_len(&mut self, value: u8) {
// //         self.buffer[1] = value;
// //     }

// //     pub fn rate(&self) -> f64 {
// //         self.parse_float(2) // buffer[2..6] = rate.to_bytes()
// //     }

// //     pub fn set_rate(&mut self, rate: f64) {
// //         self.dump_float(2, rate);
// //     }

// //     pub fn recv(&self) -> i64 {
// //         i64::from_be_bytes([
// //             self.buffer[10],
// //             self.buffer[11],
// //             self.buffer[12],
// //             self.buffer[13],
// //             self.buffer[14],
// //             self.buffer[15],
// //             self.buffer[16],
// //             self.buffer[17],
// //         ])
// //     }

// //     pub fn set_recv(&mut self, value: i64) {
// //         self.buffer[10..18].copy_from_slice(&value.to_be_bytes());
// //     }

// //     pub fn send(&self) -> i64 {
// //         i64::from_be_bytes([
// //             self.buffer[18],
// //             self.buffer[19],
// //             self.buffer[20],
// //             self.buffer[21],
// //             self.buffer[22],
// //             self.buffer[23],
// //             self.buffer[24],
// //             self.buffer[25],
// //         ])
// //     }

// //     pub fn set_send(&mut self, value: i64) {
// //         self.buffer[18..26].copy_from_slice(&value.to_be_bytes());
// //     }

// //     pub fn data_start(&self) -> usize {
// //         (self.name_len() + 26) as usize // buffer[buffer[1]+26] = data_len
// //     }

// //     pub fn data_len(&self) -> usize {
// //         let data_start = self.data_start();
// //         self.buffer[data_start] as usize // buffer[buffer[1]+27..buffer[buffer[1]+26]+buffer[1]+27] = data
// //     }

// //     pub fn dump_sender(&mut self, name: &str) {
// //         let bytes = name.as_bytes();
// //         self.set_name_len(name.len() as u8);
// //         let data_start = self.data_start();
// //         self.buffer[26..data_start].copy_from_slice(bytes);
// //     }

// //     pub fn sender_name(&self) -> String {
// //         match String::from_utf8(self.buffer[26..self.data_start()].to_vec()) {
// //             Ok(s) => s,
// //             Err(_) => String::new(),
// //         }
// //     }

// //     ///
// //     ///
// //     ///     String -> UdpPacket IO Functions
// //     ///
// //     ///
// //     ///

// //     pub fn dump_bytes(&mut self, idx: usize, length: usize, data: &[u8]) {
// //         self.buffer[idx + 1..length + idx + 1].copy_from_slice(data);
// //     }

// //     pub fn dump_string(&mut self, idx: usize, data: &str) -> usize {
// //         let data = data.as_bytes();
// //         let data_len = data.len();
// //         self.buffer[idx] = data_len as u8;
// //         self.buffer[idx + 1..data_len + idx + 1].copy_from_slice(&data);
// //         data_len + idx + 1
// //     }

// //     pub fn parse_string(&self, idx: usize) -> String {
// //         let str_len = self.buffer[idx] as usize;
// //         match String::from_utf8(self.buffer[idx + 1..str_len + idx + 1].to_vec()) {
// //             Ok(s) => s,
// //             Err(_) => String::new(),
// //         }
// //     }

// //     pub fn dump_strings(&mut self, mut idx: usize, names: &Vec<String>) {
// //         self.buffer[idx] = names.len() as u8;
// //         idx += 1;
// //         names.iter().for_each(|name| {
// //             idx = self.dump_string(idx, name);
// //         })
// //     }

// //     pub fn parse_strings(&self, mut idx: usize) -> Vec<String> {
// //         let n_data = self.buffer[idx] as usize;
// //         idx += 1;

// //         (0..n_data)
// //             .filter_map(|_| match idx < UDP_PACKET_SIZE {
// //                 true => {
// //                     let name = self.parse_string(idx);
// //                     idx += (self.buffer[idx] + 1) as usize;
// //                     match name.len() > 0 {
// //                         true => Some(name),
// //                         false => None,
// //                     }
// //                 }
// //                 false => None,
// //             })
// //             .collect()
// //     }

// //     ///
// //     ///
// //     ///     f64 -> UdpPacket IO Functions
// //     ///
// //     ///
// //     ///

// //     pub fn dump_float(&mut self, idx: usize, data: f64) -> usize {
// //         let data = data.to_be_bytes();
// //         self.buffer[idx..idx + std::mem::size_of::<f64>()].copy_from_slice(&data);
// //         idx + std::mem::size_of::<f64>()
// //     }

// //     pub fn parse_float(&self, idx: usize) -> f64 {
// //         // gross but whatever ig
// //         f64::from_be_bytes([
// //             self.buffer[idx],
// //             self.buffer[idx + 1],
// //             self.buffer[idx + 2],
// //             self.buffer[idx + 3],
// //             self.buffer[idx + 4],
// //             self.buffer[idx + 5],
// //             self.buffer[idx + 6],
// //             self.buffer[idx + 7],
// //         ])
// //     }

// //     pub fn dump_floats(&mut self, mut idx: usize, data: &Vec<f64>) {
// //         self.buffer[idx] = data.len() as u8;
// //         idx += 1;
// //         data.iter().for_each(|value| {
// //             idx = self.dump_float(idx, *value);
// //         })
// //     }

// //     pub fn parse_floats(&self, mut idx: usize) -> Vec<f64> {
// //         let n_data = self.buffer[idx] as usize;
// //         idx += 1;

// //         (0..n_data)
// //             .filter_map(|_| match idx < UDP_PACKET_SIZE {
// //                 true => {
// //                     let float = self.parse_float(idx);
// //                     idx += std::mem::size_of::<f64>();
// //                     Some(float)
// //                 }
// //                 false => None,
// //             })
// //             .collect()
// //     }

// //     ///
// //     ///
// //     ///     SocketAddr -> UdpPacket IO Functions
// //     ///
// //     ///
// //     ///

// //     pub fn dump_addr(&mut self, idx: usize, addr: &SocketAddr) -> usize {
// //         let mut octets = match addr.ip() {
// //             IpAddr::V4(ip) => ip.octets().to_vec(),
// //             _ => {
// //                 vec![0, 0, 0, 0]
// //             }
// //         };
// //         octets.append(&mut addr.port().to_be_bytes().to_vec());
// //         self.buffer[idx..idx + 6].copy_from_slice(&octets);
// //         idx + 6
// //     }

// //     pub fn parse_addr(&self, idx: usize) -> SocketAddr {
// //         sock_uri!(
// //             self.buffer[idx],
// //             self.buffer[idx + 1],
// //             self.buffer[idx + 2],
// //             self.buffer[idx + 3],
// //             u16::from_be_bytes([self.buffer[idx + 4], self.buffer[idx + 5]])
// //         )
// //     }

// //     pub fn dump_addrs(&mut self, mut idx: usize, ips: &Vec<SocketAddr>) {
// //         self.buffer[idx] = ips.len() as u8;
// //         idx += 1;
// //         ips.iter().for_each(|ip| {
// //             idx = self.dump_addr(idx, ip);
// //         });
// //     }

// //     pub fn parse_addrs(&self, mut idx: usize) -> Vec<SocketAddr> {
// //         let n_addrs = self.buffer[idx] as usize;
// //         idx += 1;

// //         (0..n_addrs)
// //             .map(|_| {
// //                 let i = idx;
// //                 idx += 6;
// //                 self.parse_addr(i)
// //             })
// //             .collect()
// //     }

// //     ///
// //     ///
// //     ///     UdpPacket Builder Functions
// //     ///
// //     ///
// //     ///

// //     pub fn packet(sock: &Sockage, mode: u8) -> SockBuffer {
// //         let mut packet = SockBuffer::new();
// //         packet.set_mode(mode);
// //         packet.dump_sender(&sock.name);
// //         packet.set_rate(
// //             (sock.send_count - sock.server_packets) as f64
// //                 / (1E-3 * (sock.socks.len() as u128 * sock.lifetime.elapsed().as_millis()) as f64),
// //         );
// //         packet.set_recv(sock.recv_count);
// //         packet.set_send(sock.send_count);
// //         packet
// //     }

// //     pub fn stamp_packet(sock: &Sockage) -> UdpPacket {
// //         SockBuffer::packet(sock, 0).buffer
// //     }

// //     pub fn target_packet(sock: &Sockage, names: &Vec<String>) -> UdpPacket {
// //         let mut packet = SockBuffer::packet(sock, 1);
// //         packet.dump_strings(packet.data_start(), names);
// //         packet.buffer
// //     }

// //     pub fn names_packet(sock: &Sockage, names: &Vec<String>) -> UdpPacket {
// //         let mut packet = SockBuffer::packet(sock, 2);
// //         packet.dump_strings(packet.data_start(), names);
// //         packet.buffer
// //     }

// //     pub fn addrs_packet(sock: &Sockage, addrs: &Vec<SocketAddr>) -> UdpPacket {
// //         let mut packet = SockBuffer::packet(sock, 3);
// //         packet.dump_addrs(packet.data_start(), addrs);
// //         packet.buffer
// //     }

// //     pub fn data_packet(sock: &Sockage, data: &Vec<f64>) -> UdpPacket {
// //         let mut packet = SockBuffer::packet(sock, 4);
// //         packet.dump_floats(packet.data_start(), data);
// //         packet.buffer
// //     }

// //     pub fn parse_name_packet(&self) -> (String, Vec<String>) {
// //         (self.sender_name(), self.parse_strings(self.data_start()))
// //     }

// //     pub fn parse_addr_packet(&self) -> (String, Vec<SocketAddr>) {
// //         (self.sender_name(), self.parse_addrs(self.data_start()))
// //     }

// //     pub fn parse_data_packet(&self) -> (String, Vec<f64>) {
// //         (self.sender_name(), self.parse_floats(self.data_start()))
// //     }
// // }
