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
use more_asserts::assert_le;
// use crate::socks::socks::*;
// use crate::socks::data_structures::*;
use std::{
    io,
    env,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use socket2::{Domain, Protocol, Socket, Type};


macro_rules! ipv4 {
    () => {{
        Ipv4Addr::new(0, 0, 0, 0)
    }};
    ($ip1:expr, $ip2:expr, $ip3:expr, $ip4:expr) => {{
        Ipv4Addr::new($ip1, $ip2, $ip3, $ip4)
    }};
}

macro_rules! sock_uri {
    () => {{
        SocketAddr::new(IpAddr::V4(ipv4!(0, 0, 0, 0)), 0)
    }};
    ($port:expr) => {{
        SocketAddr::new(IpAddr::V4(ipv4!(0, 0, 0, 0)), $port)
    }};
    ($ip1:expr, $ip2:expr, $ip3:expr, $ip4:expr, $port:expr) => {{
        SocketAddr::new(IpAddr::V4(ipv4!($ip1, $ip2, $ip3, $ip4)), $port)
    }};
}

fn new_multicast(port: u16) -> UdpSocket {

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    socket.set_nonblocking(true).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    socket.set_write_timeout(Some(Duration::from_millis(100))).unwrap();
    socket.join_multicast_v4(&ipv4!(224, 0, 0, 251), &Ipv4Addr::UNSPECIFIED).expect("could not join multicast");
    socket.bind(&sock_uri!(port).into()).unwrap();

    socket.try_into().unwrap()
}

fn new_sender() -> UdpSocket {

    let socket = UdpSocket::bind(sock_uri!()).unwrap();

    socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    socket.set_write_timeout(Some(Duration::from_millis(100))).unwrap();

    socket
}

#[cfg(test)]
pub mod usock {
    use super::*;
    pub const UDP_PACKET_SIZE: usize = 1024;

    #[test]
    pub fn multicast_node() {

        let lifetime = Instant::now();
        let socket = new_sender();

        let mut buffer = [255u8; UDP_PACKET_SIZE];
        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();

            socket.send_to(&mut buffer, sock_uri!(224, 0, 0, 251, 1313)).unwrap();

            while t.elapsed().as_millis() < 500 {}
        }
    }

    #[test]
    pub fn multicast_listen() {

        let lifetime = Instant::now();
        let listener = new_multicast(1313);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            let mut buffer = [0u8; UDP_PACKET_SIZE];

            match listener.recv_from(&mut buffer) {
                Ok((_, src)) => {
                    println!("listener received data from {}", src);
                }
                _ => {},
            }

            while t.elapsed().as_micros() < 100 {}
        }
    }

    #[test]
    pub fn multicast_listen2() {

        let lifetime = Instant::now();
        let listener = new_multicast(1313);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            let mut buffer = [0u8; UDP_PACKET_SIZE];

            match listener.recv_from(&mut buffer) {
                Ok((_, src)) => {
                    println!("listener2 received data from {}", src);
                }
                _ => {},
            }

            while t.elapsed().as_micros() < 100 {}
        }
    }
}

// #[cfg(test)]
// pub mod sockbuffer {
//     use super::*;

//     #[test]
//     pub fn sock_io() {
//         let mut buffer = SockBuffer::new();

//         let name = "name";
//         let mode = 13;
//         let connect = 1;
//         let send_count = 2;
//         let recv_count = 2;
//         let rate = 3.0;
//         let timestamp = 4.0;

//         buffer.set_sock(
//             name.as_bytes(),
//             mode,
//             connect,
//             send_count,
//             recv_count,
//             rate,
//             timestamp,
//         );

//         let (
//             new_name,
//             new_mode,
//             new_connect,
//             new_send_count,
//             new_recv_count,
//             new_rate,
//             new_timestamp,
//         ) = buffer.get_sock();

//         assert_eq!(new_name, name, "name is wrong");
//         assert_eq!(new_mode, mode, "mode is wrong");
//         assert_eq!(new_connect, connect, "connect is wrong");
//         assert_eq!(new_send_count, send_count, "send is wrong");
//         assert_eq!(new_recv_count, recv_count, "recv is wrong");
//         assert_eq!(new_rate, rate, "rate is wrong");
//         assert_eq!(new_timestamp, timestamp, "timestamp is wrong");
//     }

//     #[test]
//     pub fn names_io() {
//         let mut buffer = SockBuffer::new();

//         let targets = vec![
//             "name1".to_string(),
//             "name2".to_string(),
//             "name3".to_string(),
//             "name4".to_string(),
//         ];

//         buffer.set_names(&targets);

//         let new_targets = buffer.get_names();

//         (0..targets.len()).for_each(|i| {
//             assert_eq!(targets[i], new_targets[i], "wrong target");
//         })
//     }

//     #[test]
//     pub fn target_io() {
//         let mut buffer = SockBuffer::new();

//         let targets = vec![
//             "name1".to_string(),
//             "name2".to_string(),
//             "name3".to_string(),
//             "name4".to_string(),
//         ];
//         let addrs = vec![
//             sock_uri!(1313),
//             sock_uri!(1313),
//             sock_uri!(1313),
//             sock_uri!(1313),
//         ];

//         buffer.set_targets(&targets, &addrs);

//         let (new_targets, new_addrs) = buffer.get_targets();

//         (0..targets.len()).for_each(|i| {
//             assert_eq!(targets[i], new_targets[i], "wrong target");
//             assert_eq!(addrs[i], new_addrs[i], "wrong addrs");
//         })
//     }
// }

// #[cfg(test)]
// pub mod message {
//     use super::*;

//     #[test]
//     pub fn message_shatter() {
//         let payload = (0..255).collect();
//         let message = Message::from_payload(0, payload);
//         let packets = message.fragment_bytes();

//         let mut new_message = Message::new();
//         packets.into_iter().for_each(|packet| new_message.collect(packet));

//         let new_payload = new_message.to_payload();

//         assert_eq!((0..255).collect::<Vec<u8>>(), new_payload, "returned the wrong payload");
//     }
// }

// #[cfg(test)]
// pub mod u_socks {
//     use super::*;
//     pub const UDP_PACKET_SIZE: usize = 1024;

//     #[test]
//     pub fn multicast_read() {
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
//     pub fn multicast_write() {
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
// }