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

use crate::{ipv4, sock_uri, socks::data_structures::*, socks::socks::*};
use std::{
    env, io,
    time::{Duration, Instant},
};

#[cfg(test)]
pub mod usock {
    use super::*;
    pub const UDP_PACKET_SIZE: usize = 1024;

    #[test]
    pub fn multicast_node() {
        let lifetime = Instant::now();
        let mut sock = Sock::new("node1", vec![]);

        let msg = Message::from_payload(sock.header_bytes(), vec![1; 2048]);

        assert_eq!(msg.fragments.len(), 3);
        assert_eq!(msg.fragments[0].n_bytes, 950);
        assert_eq!(msg.fragments[1].n_bytes, 950);
        assert_eq!(msg.fragments[2].n_bytes, 148);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            msg.as_packets().iter().for_each(|buffer| {
                sock.tx(*buffer, MULTICAST_URI);
            });

            while t.elapsed().as_millis() < 500 {}
        }
    }

    #[test]
    pub fn multicast_listen() {
        let mut sock = Sock::new("node2", vec!["node1"]);

        while sock.lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            let mut buffer = [0u8; UDP_PACKET_SIZE];
            match sock.try_rx(&mut buffer) {
                Some(i) => {
                    println!(
                        "listener: received {} bytes from {:?}",
                        sock.messages[i].to_payload().len(),
                        sock.targets[i],
                    )
                }
                _ => {}
            };

            while t.elapsed().as_micros() < 100 {}
        }
    }
}

#[cfg(test)]
pub mod message {
    use super::*;

    #[test]
    pub fn message_shatter() {
        let payload = (0..255).collect();
        let message = Message::from_payload([0; SOCK_HEADER_LEN], payload);
        let packets = message.as_packets();

        let mut new_message = Message::new();

        packets.into_iter().for_each(|packet| {
            new_message.collect(MessageFragment::from_bytes(packet).1);
        });

        let new_payload = new_message.to_payload();

        assert_eq!(
            (0..255).collect::<Vec<u8>>(),
            new_payload[0..255],
            "returned the wrong payload"
        );
    }

    #[test]
    pub fn message_from_sock() {
        let sock = Sock::new("node1", vec![]);
        let msg = Message::from_payload(sock.header_bytes(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let (name, _, _, _) = Sock::header_from_bytes(&msg.header);

        assert_eq!(name, "node1", "name was wrong");

        let packets = msg.as_packets();
        let (header, _) = MessageFragment::from_bytes(packets[0]);
        let (name1, _, _, _) = Sock::header_from_bytes(&header);

        assert_eq!(name1, "node1", "name1 was wrong");
    }
}

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
