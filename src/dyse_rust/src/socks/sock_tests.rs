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

use crate::{ipv4, sock_uri, socks::message::*, socks::sockapi::*, socks::socks::*};
use std::{
    env, io,
    time::{Duration, Instant},
};

#[cfg(test)]
pub mod usock {
    use super::*;
    pub const UDP_PACKET_SIZE: usize = 1024;

    #[test]
    pub fn multicast_sender() {
        let lifetime = Instant::now();
        let mut sock = Sock::new("node1", vec![]);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            let msg = Message::from_payload(vec![1; 100]);
            msg.packets(sock.header_bytes()).iter().for_each(|buffer| {
                sock.tx(*buffer, MULTICAST_URI);
            });

            while t.elapsed().as_millis() < 500 {}
        }

        sock.log_heavy("");
    }

    #[test]
    pub fn multicast_listener() {
        let mut sock = Sock::new("node2", vec!["node1".to_string()]);

        while sock.lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            let mut buffer = [0u8; UDP_PACKET_SIZE];
            match sock.try_rx(&mut buffer) {
                Some(i) => {
                    sock.log(format!(
                        "received {} bytes from {:?}",
                        sock.messages[i].to_payload().len(),
                        sock.targets[i],
                    ));
                }
                _ => {}
            };

            while t.elapsed().as_micros() < 100 {}
        }

        sock.log_heavy("");
    }

    #[test]
    pub fn multicast_listener2() {
        let mut sock = Sock::new("node3", vec!["node1".to_string()]);

        while sock.lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            let mut buffer = [0u8; UDP_PACKET_SIZE];
            match sock.try_rx(&mut buffer) {
                Some(i) => {
                    sock.log(format!(
                        "received {} bytes from {:?}",
                        sock.messages[i].to_payload().len(),
                        sock.targets[i],
                    ));
                }
                _ => {}
            };

            while t.elapsed().as_micros() < 100 {}
        }

        sock.log_heavy("");
    }
}

#[cfg(test)]
pub mod low_sock {
    use super::*;
    pub const UDP_PACKET_SIZE: usize = 1024;

    #[test]
    pub fn node0() {
        let lifetime = Instant::now();
        let mut sock = Sock::new("node0", vec![]);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            sock.tx_payload(vec![1; 100]);
            while t.elapsed().as_millis() < 250 {}
        }

        sock.log_heavy("");
    }

    pub fn multicast_receiver(name: &str, targets: Vec<String>, callback: SockClosureFn) {
        let mut sock = Sock::new(name, vec![]);
        sock.link_closure(&targets, callback);
        sock.spin();
        sock.log_heavy("");
    }

    #[test]
    pub fn node1() {
        multicast_receiver("node1", vec!["node0".to_string()], |data, output, _| {
            *output = data;
            0
        });
    }

    #[test]
    pub fn node2() {
        multicast_receiver("node2", vec!["node0".to_string(), "node1".to_string()], |data, _, _| {
            println!("[node0, node1]: {} {data:?}", data.len());
            0
        });
    }
}

#[cfg(test)]
pub mod mid_sock {
    use super::*;

    #[test]
    pub fn node0() {
        let lifetime = Instant::now();
        let mut sock = Sock::new("node0", vec![]);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            sock.tx_payload((1..11).collect());
            while t.elapsed().as_millis() < 10 {}
        }

        sock.log_heavy("");
    }

    pub fn multicast_receiver(name: &str, targets: Vec<String>, callback: SockClosureFn) {
        let mut sock = SockApi::relay(name, &targets, callback);
        sock.spin();
        sock.log_heavy("");
    }

    #[test]
    pub fn relay() {
        multicast_receiver("node1", vec!["node0".to_string()], |data, output, _| {
            *output = data.iter().map(|x| x + 1).collect();
            0
        });
    }

    // #[test]
    // pub fn echo() {
    //     SockApi::echo(&vec!["node0", "node1"]);
    // }

    #[test]
    pub fn hz() {
        SockApi::hz(&vec!["node0".to_string(), "node1".to_string()]);
    }
}

#[cfg(test)]
pub mod high_sock {
    use super::*;

    #[test]
    pub fn node0() {
        let lifetime = Instant::now();
        let mut sock = Sock::new("node0", vec![]);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            sock.tx_payload((1..11).collect());
            while t.elapsed().as_millis() < 10 {}
        }

        sock.log_heavy("");
    }

    #[test]
    pub fn node1() {
        let lifetime = Instant::now();
        let mut sock = Sock::new("node1", vec![]);

        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            sock.tx_payload((1..11).collect());
            while t.elapsed().as_millis() < 10 {}
        }

        sock.log_heavy("");
    }

    pub fn multicast_hub(name: &str, targets: Vec<Vec<String>>, callback: Vec<SockClosureFn>) {
        let mut sock = SockApi::hub(name, targets, callback);
        sock.spin();
        sock.log_heavy("");
    }

    #[test]
    pub fn relay() {
        let targets = vec![vec!["node0".to_string()], vec!["node1".to_string()]];
        let closures = vec![
            |_: Vec<u8>, _: &mut Vec<u8>, _| {
                // *output = data.into_iter().map(|x| x).collect();
                0
            }, 
            |data: Vec<u8>, output: &mut Vec<u8>, _| {
                *output = data.into_iter().map(|x| x).collect();
                0
            }
        ];

        multicast_hub("node2",targets, closures);
    }

    #[test]
    pub fn hz() {
        SockApi::hz(&vec!["node2".to_string()]);
    }
}

#[cfg(test)]
pub mod message {
    use super::*;

    #[test]
    pub fn message_shatter() {
        let payload = (0..255).collect();
        let message = Message::from_payload(payload);
        let packets = message.packets([0; SOCK_HEADER_LEN]);

        let mut new_message = Message::new();

        packets.into_iter().for_each(|packet| {
            new_message.collect(0, 0, MessageFragment::from_bytes(packet).1);
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
        let msg = Message::from_payload(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let packets = msg.packets([0; SOCK_HEADER_LEN]);
        let (header, _) = MessageFragment::from_bytes(packets[0]);
        let (name1, _, _, _) = Sock::header_from_bytes(header);

        assert_eq!(name1, "node1", "name1 was wrong");

        let big_msg = Message::from_payload(vec![1; 2048]);

        assert_eq!(big_msg.fragments.len(), 3);
        assert_eq!(big_msg.fragments[0].n_bytes, 950);
        assert_eq!(big_msg.fragments[1].n_bytes, 950);
        assert_eq!(big_msg.fragments[2].n_bytes, 148);
    }
}
