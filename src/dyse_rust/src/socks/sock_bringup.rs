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
#![allow(unused_imports)]
#![allow(unused_macros)]
use crate::socks::socks::*;
use crate::socks::data_structures::*;
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use more_asserts::assert_le;

macro_rules! sock_uri {
    () => {{
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)
    }};
    ($port:expr) => {{
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), $port)
    }};
    ($port:expr, $ip1:expr, $ip2:expr, $ip3:expr, $ip4:expr) => {{
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new($ip1, $ip2, $ip3, $ip4)), $port)
    }};
}

pub fn packet_sum(buffer: [u8; UDP_PACKET_SIZE]) -> u32 {
    buffer.iter().map(|x| *x as u32).sum()
}

// #[cfg(test)]
// pub mod u_socks {
//     use super::*;
//     pub const UDP_PACKET_SIZE: usize = 1024;

//     #[test]
//     pub fn init_read() {
//         let mut t = Instant::now();
//         let socket = UdpSocket::bind(env::var("DYSE_CORE_URI").unwrap())
//             .expect("Couldn't bind to socket 1313");
//         socket.set_read_timeout(Some(Duration::new(0, 1))).unwrap();
//         assert_le!(t.elapsed().as_micros(), 120, "Core bind time us");

//         t = Instant::now();
//         let mut buf = [0; UDP_PACKET_SIZE];
//         let (size, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
//         assert_le!(t.elapsed().as_micros(), 123, "Core read time us");

//         assert_eq!(
//             src_addr,
//             SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878)
//         );
//         assert_eq!(size, UDP_PACKET_SIZE, "Packet was incorrect size");
//         assert_eq!(
//             packet_sum(buf),
//             UDP_PACKET_SIZE as u32,
//             "Buffer read had incorrect sum"
//         );
//     }

//     #[test]
//     pub fn init_write() {
//         let mut t = Instant::now();
//         let socket = UdpSocket::bind("127.0.0.1:7878").expect("Couldn't bind to socket 7878");
//         assert_le!(t.elapsed().as_micros(), 100, "Client bind time us");

//         t = Instant::now();
//         socket.connect(env::var("DYSE_CORE_URI").unwrap()).unwrap();
//         socket.set_write_timeout(Some(Duration::new(0, 1))).unwrap();
//         assert_le!(t.elapsed().as_micros(), 100, "Client connect time us");

//         t = Instant::now();
//         let mut buf = [1; UDP_PACKET_SIZE];
//         let size = socket.send(&mut buf).expect("Didn't send data");
//         assert_le!(t.elapsed().as_micros(), 100, "Client write time us");

//         assert_eq!(size, UDP_PACKET_SIZE, "Sent the wrong packet size");
//     }

//     #[test]
//     pub fn init_read_loop() {
//         let millis = 500;
//         let lifetime = Instant::now();
//         while lifetime.elapsed().as_micros() < 250 as u128 {}

//         let mut lt = Instant::now();
//         let socket = UdpSocket::bind(env::var("DYSE_CORE_URI").unwrap())
//             .expect("Couldn't bind to socket 1313");
//         socket.set_read_timeout(Some(Duration::new(0, 1))).unwrap();
//         assert_le!(lt.elapsed().as_micros(), 100, "Core bind time us");

//         assert_eq!(
//             socket.local_addr().unwrap(),
//             SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
//             "Didn't bind to requested IP"
//         );

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 1 {
//             lt = Instant::now();
//             let mut buf = [0; UDP_PACKET_SIZE];
//             let (size, src_addr) = match socket.recv_from(&mut buf) {
//                 Ok((size, src_addr)) => (size, src_addr),
//                 Err(e) => {
//                     println!("Read {:?}", e);
//                     (
//                         0,
//                         SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
//                     )
//                 }
//             };

//             assert_le!(lt.elapsed().as_micros(), 500, "Client read time us");

//             assert_eq!(
//                 src_addr,
//                 SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878)
//             );
//             assert_eq!(size, UDP_PACKET_SIZE, "Packet size was incorrect size");
//             assert_eq!(
//                 packet_sum(buf),
//                 UDP_PACKET_SIZE as u32,
//                 "Buffer read had incorrect sum"
//             );

//             while t.elapsed().as_millis() < millis {}
//         }
//         println!("R Finished {}s", t.elapsed().as_micros() as f64 * 1E-6);
//     }

//     #[test]
//     pub fn init_write_loop() {
//         let millis = 500;
//         let lifetime = Instant::now();
//         while lifetime.elapsed().as_micros() < 250 as u128 {}

//         let mut lt = Instant::now();
//         let socket = UdpSocket::bind("127.0.0.1:7878").expect("Couldn't bind to socket 7878");
//         socket.connect(env::var("DYSE_CORE_URI").unwrap()).unwrap();
//         socket.set_write_timeout(Some(Duration::new(0, 1))).unwrap();
//         assert_le!(lt.elapsed().as_micros(), 100, "Client bind time us");

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 1 {
//             lt = Instant::now();
//             let mut buf = [1; UDP_PACKET_SIZE];
//             let size = match socket.send(&mut buf) {
//                 Ok(size) => size,
//                 _ => 0, // broken connection, session died
//             };
//             assert_le!(lt.elapsed().as_micros(), 500, "Client send time us");

//             match size {
//                 0 => break,
//                 _ => {}
//             }

//             assert_eq!(size, UDP_PACKET_SIZE, "Sent wrong packet size");

//             while t.elapsed().as_millis() < millis {}
//         }
//         println!("W Finished {}s", t.elapsed().as_micros() as f64 * 1E-6);
//     }

//     #[test]
//     pub fn init_read_loop_obj() {
//         let millis = 500;
//         let lifetime = Instant::now();
//         while lifetime.elapsed().as_secs() < 0 as u64 {}

//         let mut lt = Instant::now();
//         let mut sock = Sockage::core("loop_reader");
//         assert_le!(lt.elapsed().as_micros(), 100, "Core bind time us");

//         assert_eq!(
//             sock.socket.local_addr().unwrap(),
//             SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
//             "Didn't bind to requested IP"
//         );

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 1 {
//             lt = Instant::now();
//             let mut buffer = [0; UDP_PACKET_SIZE];
//             sock.recv(&mut buffer);

//             assert_le!(lt.elapsed().as_micros(), 500, "Client read time us");
//             assert_eq!(
//                 packet_sum(buffer),
//                 UDP_PACKET_SIZE as u32,
//                 "Buffer read had incorrect sum"
//             );

//             while t.elapsed().as_millis() < millis {}
//         }

//         println!(
//             "Core Finished {}s",
//             lifetime.elapsed().as_micros() as f64 * 1E-6
//         );
//     }

//     #[test]
//     pub fn init_write_loop_obj() {
//         let millis = 500;
//         let lifetime = Instant::now();
//         while lifetime.elapsed().as_secs() < 0 as u64 {}

//         let mut lt = Instant::now();
//         let mut sock = Sockage::new("loop_writer", sock_uri!(7878), millis);
//         assert_le!(lt.elapsed().as_micros(), 100, "Client bind time us");

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 1 {
//             lt = Instant::now();
//             let buffer = [1; UDP_PACKET_SIZE];
//             sock.send_to(buffer, sock_uri!(1313));
//             assert_le!(lt.elapsed().as_micros(), 500, "Client send time us");

//             while t.elapsed().as_millis() < millis {}
//         }
//         println!(
//             "Client Finished {}s",
//             lifetime.elapsed().as_micros() as f64 * 1E-6
//         );
//     }
// }

// #[cfg(test)]
// pub mod low_socks {
//     use super::*;

//     #[test]
//     pub fn core() {
//         // let millis = 10;
//         let lifetime = Instant::now();
//         while lifetime.elapsed().as_secs() < 0 as u64 {}

//         let mut known_names: Vec<String> = vec![];
//         let mut known_addrs: Vec<SocketAddr> = vec![];

//         let mut sock = Sockage::core("low_sock");

//         assert_eq!(
//             sock.socket.local_addr().unwrap(),
//             SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
//             "Core Didn't bind to requested IP"
//         );

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 3 {
//             let mut buffer = SockBuffer::new();
//             let src = sock.recv(&mut buffer.buffer);

//             sock.shutdown(false);

//             match src.port() > 0 {
//                 true => {
//                     let name = buffer.sender_name();
//                     match known_addrs.iter().find(|a| **a == src) {
//                         Some(_) => {}
//                         _ => {
//                             sock.log(format!("New Request from {}", name));
//                             known_addrs.push(src);
//                             known_names.push(name);
//                         }
//                     }
//                     sock.send_to(SockBuffer::stamp_packet(&sock), src);
//                 }

//                 _ => {}
//             }

//             // while t.elapsed().as_millis() < millis {}
//         }

//         sock.log(format!(
//             "Finished: ({},{}) packet/s (in/out)",
//             sock.recv_count as u64 / lifetime.elapsed().as_secs(),
//             sock.send_count as u64 / lifetime.elapsed().as_secs(),
//         ));
//         sock.log(format!("Ports: {:?}", known_addrs));
//         sock.log(format!("Names: {:?}", known_names));

//         sock.log(format!(
//             "Finished {}s",
//             lifetime.elapsed().as_micros() as f64 * 1E-6
//         ));
//     }

//     pub fn simple_node(name: &str, _: f64) {
//         // let millis = (1000.0 / rate) as u128;
//         let mut lifetime = Instant::now();
//         while lifetime.elapsed().as_millis() < 1 as u128 {}

//         let mut core_replies = 0;
//         let mut sock = Sockage::client(name);

//         sock.log("Active");

//         lifetime = Instant::now();
//         let mut t = Instant::now();
//         while lifetime.elapsed().as_secs() < 3 {
//             sock.send_to(SockBuffer::stamp_packet(&sock), sock_uri!(1313));

//             let mut buffer = SockBuffer::new();
//             let src = sock.recv(&mut buffer.buffer);

//             match src.port() {
//                 1313 => {
//                     // let name = buffer.sender_name();
//                     // sock.log(format!("Request from {}", name));
//                     core_replies += 1;
//                 }

//                 _ => {}
//             }

//             while t.elapsed().as_micros() < sock.micros_rate * 100 {}
//             t = Instant::now();
//         }

//         sock.log(format!(
//             "Finished: ({},{}) packet/s (in/out)",
//             sock.recv_count as u64 / lifetime.elapsed().as_secs(),
//             sock.send_count as u64 / lifetime.elapsed().as_secs(),
//         ));
//         assert_le!(1, core_replies, "Didn't receive Reply from server");
//     }

//     #[test]
//     pub fn node1() {
//         simple_node("sock_node1", 2.0);
//     }

//     #[test]
//     pub fn node2() {
//         simple_node("sock_node2", 4.0);
//     }
// }

// #[cfg(test)]
// pub mod mid_socks {
//     use super::*;

//     #[test]
//     pub fn core() {
//         let mut sock = Sockage::core("mid_sock");

//         assert_eq!(
//             sock.socket.local_addr().unwrap(),
//             SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
//             "Core Didn't bind to requested IP"
//         );

//         // let t = Instant::now();
//         while sock.lifetime.elapsed().as_secs() < 6 {
//             let mut buffer = SockBuffer::new();
//             let src = sock.recv(&mut buffer.buffer);

//             match (buffer.mode(), src.port() > 0) {
//                 (0, true) => {
//                     // sock.send_to(SockBuffer::stamp_packet(&sock.name), src);
//                     // sock.log(format!("Stamp Request {:?}", src));
//                 }
//                 (1, true) => {
//                     let (sender, names) = buffer.parse_name_packet();
//                     sock.discover_sock(&sender, &src);

//                     // sock.log(format!("New Receiver {}", sender));

//                     sock.add_target_to_names(&names, sender);
//                     sock.send_to(SockBuffer::stamp_packet(&sock), src); // reply timestamp to node registering as receiver

//                     // (0..names.len()).for_each(|i| sock.send_target_addresses(&names[i])); // send data to nodes registered as sender
//                 }
//                 (2, true) => {
//                     sock.log(format!("Received 2 from {:?}", src));
//                 }
//                 (3, true) => {
//                     sock.log(format!("Received 3 from {:?}", src));
//                 }
//                 (4, true) => {
//                     let sender = buffer.sender_name();
//                     // sock.log(format!("New Sender: {}", sender));
//                     // sock.send_to(SockBuffer::stamp_packet(&sock.name), src);
//                     sock.discover_sock(&sender, &src);

//                     sock.send_target_names(&sender); // send data to nodes registered as sender
//                     sock.send_target_addresses(&sender); // send data to nodes registered as sender
//                 }
//                 _ => {}
//             };

//             // while t.elapsed().as_millis() < 10 {}
//         }

//         sock.log_heavy();
//     }

//     pub fn simple_node(name: &str, rate: f64, subscribers: Vec<String>) {
//         let mut sock = Sockage::client(name);

//         let millis = (1E6 / rate) as u128;

//         if rate > 0.0 {
//             sock.micros_rate = millis;
//             sock.send_to(
//                 SockBuffer::data_packet(&sock, &vec![]),
//                 sock_uri!(1313),
//             );
//         } else {
//             sock.micros_rate = 250;
//         }

//         if subscribers.len() > 0 {
//             sock.data_request(&subscribers);
//         }

//         sock.log("Active");

//         let mut t = Instant::now();
//         while sock.lifetime.elapsed().as_secs() < 3 {
//             let mut buffer = SockBuffer::new();

//             match rate > 0.0 && t.elapsed().as_micros() >= sock.micros_rate {
//                 true => {
//                     sock.data_broadcast(&vec![1.0, 2.0, 3.0, 4.0]);
//                     t = Instant::now();
//                 }
//                 _ => {}
//             };

//             let src = sock.recv(&mut buffer.buffer);

//             match (src.port(), buffer.mode()) {
//                 (0, _) => {}
//                 (1313, 0) => {
//                     // sock.log("Received stamp");
//                 }
//                 (1313, 1) => {
//                     let (_, names) = buffer.parse_name_packet();
//                     sock.register_names(&names);
//                     // sock.log(format!("Received names {:?}", names));
//                 }
//                 (1313, 2) => {
//                     let (_, addrs) = buffer.parse_addr_packet();

//                     sock.register_addrs(&addrs);
//                     if !sock.targets_configured {
//                         // if addresses fails request new data
//                         sock.send_to(
//                             SockBuffer::data_packet(&sock, &vec![]),
//                             sock_uri!(1313),
//                         );
//                     }
//                     // sock.log(format!("Received addresses {:?}", addrs));
//                 }
//                 (_, 4) => {

//                     // let (sender, data) = buffer.parse_data_packet();
//                     // sock.log(format!("Received data from {} {:?}", sender, data));
//                 }
//                 _ => {}
//             }
//         }

//         sock.log_heavy();
//         if rate > 0.0 {
//             assert_le!(
//                 (0.85 * rate * sock.lifetime.elapsed().as_secs() as f64),
//                 sock.send_count as f64,
//                 "Not enough packts sent"
//             );
//         }

//         if subscribers.len() > 0 {
//             assert_le!(
//                 (850.0 * sock.lifetime.elapsed().as_secs() as f64),
//                 sock.recv_count as f64,
//                 "Not enough packts recv"
//             );
//         }
//     }

//     #[test]
//     pub fn node1() {
//         let t = Instant::now();
//         while t.elapsed().as_millis() < 1 as u128 {}
//         simple_node("sock_node1", 1000.0, vec![]);
//     }

//     #[test]
//     pub fn node2() {
//         simple_node("sock_node2", 1000.0, vec!["sock_node1".to_string()]);
//     }

//     #[test]
//     pub fn node3() {
//         simple_node(
//             "sock_node3",
//             0.0,
//             vec!["sock_node1".to_string(), "sock_node2".to_string()],
//         );
//     }

//     #[test]
//     pub fn node4() {
//         // let mut t = Instant::now();
//         // while t.elapsed().as_millis() < 4 as u128 {}
//         simple_node("sock_node4", 3000.0, vec!["sock_node4".to_string()]);
//     }

//     #[test]
//     pub fn node5() {
//         let t = Instant::now();
//         while t.elapsed().as_millis() < 500 as u128 {}
//         simple_node("sock_node5", 3000.0, vec!["sock_node5".to_string()]);
//     }
// }

#[cfg(test)]
pub mod high_socks {
    use super::*;

    #[test]
    pub fn core() {
        let mut sock = Sockage::core("high_sock");

        assert_eq!(
            sock.socket.local_addr().unwrap(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
            "Core Didn't bind to requested IP"
        );

        while sock.lifetime.elapsed().as_secs() < 6 {
            sock.core_parse();
        }

        sock.send_terminate();
        sock.log_heavy();
    }

    #[test]
    pub fn node1() {
        let mut t = Instant::now();
        while t.elapsed().as_millis() < 1 as u128 {}
        let mut sock = Sockage::sender("sock_node1");
        sock.micros_rate = 5000;

        let mut v = 0.0;

        while sock.lifetime.elapsed().as_secs() < 5 {
            sock.send(vec![v, v, v, v, v]);
            v = v + 1.0;
            while t.elapsed().as_micros() <= sock.micros_rate {}
            t = Instant::now();
        }

        assert_eq!(sock.targets_configured, true, "Failed to configure targets");
        sock.log_heavy();
    }

    #[test]
    pub fn node2() {
        let names = vec!["sock_node1".to_string()];

        pub fn callback(sock: &mut Sockage, _: &Vec<String>) {
            let data = sock.data[0].iter().map(|x| -x).collect();
            sock.data_broadcast(&data);
        }

        let handle = Sockage::thread("sock_node2".to_string(), names, &callback);

        handle.join().expect("sock_node2 thread failed");
    }

    #[test]
    pub fn node3() {
        Sockage::echo(vec!["sock_node1".to_string(), "sock_node2".to_string()]);
    }
}
