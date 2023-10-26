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
use std::thread::{Builder, JoinHandle};

use crate::socks::socks::*;

pub struct SockApi {}

impl SockApi {
    pub fn sender(name: &str) -> Sock {
        Sock::new(name, vec![])
    }

    pub fn receiver(name: &str, targets: Vec<String>) {
        let mut sock = Sock::new(name, targets);
        sock.spin();
    }

    pub fn relay(name: &str, targets: &Vec<String>, callback: SockClosureFn) -> Sock {
        let mut sock = Sock::new(name, vec![]);
        sock.link_closure(targets, callback);
        sock
    }

    pub fn hub(name: &str, targets: Vec<Vec<String>>, callbacks: Vec<SockClosureFn>) -> Sock {
        let mut sock = Sock::new(name, vec![]);
        (0..targets.len()).for_each(|i| sock.link_closure(&targets[i], callbacks[i]));
        sock
    }

    pub fn relay_thread(
        name: String,
        names: &Vec<String>,
        callback: SockClosureFn,
    ) -> JoinHandle<()> {
        let mut sock = SockApi::relay(&name, names, callback);

        Builder::new()
            .name(name)
            .spawn(move || {
                sock.spin();
                sock.log_heavy("exiting");
            })
            .unwrap()
    }

    pub fn hub_thread(
        name: String,
        names: Vec<Vec<String>>,
        callbacks: Vec<SockClosureFn>,
    ) -> JoinHandle<()> {
        let mut sock = SockApi::hub(&name, names, callbacks);

        Builder::new()
            .name(name)
            .spawn(move || {
                sock.spin();
                sock.log_heavy("exiting");
            })
            .unwrap()
    }

    pub fn echo(names: &Vec<String>) {
        let mut sock = SockApi::relay("echo-sock", names, |data, _, _| {
            println!("{data:?}");
            0
        });

        sock.spin();
        sock.log_heavy("exiting");
    }

    pub fn hz(names: &Vec<String>) {
        let mut sock = SockApi::relay("hz-sock", names, move |_, _, ts| {
            println!("{:.3} Hz", 1.0 / ts);
            0
        });

        sock.spin();
        sock.log_heavy("exiting");
    }

    // pub fn shutdown_socks() {
    //     let mut sock = Sockage::client("kill-sock");
    //     let mut buffer = SockBuffer::stamp_packet(&sock);

    //     buffer[0] = 13;
    //     sock.send_to(buffer, sock_uri!(1313));
    // }
}
