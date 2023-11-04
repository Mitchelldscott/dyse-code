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

use crate::{
    ipv4, 
    sock_uri, 
    build_fn,
    sync,
    unsync,
    add_task,
    socks::{
        sockapi, 
        task::*,
        socks::*, 
        message::*, 
    }
};
use std::{
    env, io,
    time::{Duration, Instant},
};

#[cfg(test)]
pub mod high_sock {
    use super::*;

    #[test]
    pub fn demo_source() {
        let lifetime = Instant::now();
        let mut sock = Sock::source("source");

        let mut i = 0.0f64;
        while lifetime.elapsed().as_secs() < 5 {
            let t = Instant::now();
            sock.tx_payload(i);
            i += 1.0;
            while t.elapsed().as_millis() < 250 {}
        }

        sock.log_heavy("");
        sockapi::shutdown();
    }

    #[test]
    pub fn demo_hub() {
        let mut sock = unsync!("hub", vec!["source"], "inv", 0, |_ctx: u8, data: f64| {
            -data[0]
        });
        add_task!(sock, vec![], "val", 0, |_ctx: u8, data: f64| {
            data[0]
        });
        sock.spin();
        sock.log_heavy("");
    }

    #[test]
    pub fn demo_relay() {
        let mut sock = sync!("relay", vec!["val", "inv"], "sum", 2, |_ctx: u8, a: f64, b: f64| {
            a + b
        });
        sock.spin();
        sock.log_heavy("");
    }

    #[test]
    pub fn demo_hz() {
        let t = Instant::now();
        while t.elapsed().as_secs() < 3 {}
        sockapi::hz::<f64>("demo/hz", vec!["sum", "val", "inv"]);
    }

    // #[test]
    // pub fn demo_echo() {
    //     sockapi::sync_echo::<f64>("demo/echo", vec!["sum", "val", "inv"]);
    // }
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

        let sock = Sock::source("node0");
        let (name1, _, _, _) = sock.header_from_bytes(header);

        assert_eq!(name1, "node1", "name1 was wrong");

        let big_msg = Message::from_payload(vec![1; 2048]);

        assert_eq!(big_msg.fragments.len(), 3);
        assert_eq!(big_msg.fragments[0].n_bytes, 950);
        assert_eq!(big_msg.fragments[1].n_bytes, 950);
        assert_eq!(big_msg.fragments[2].n_bytes, 148);
    }
}
