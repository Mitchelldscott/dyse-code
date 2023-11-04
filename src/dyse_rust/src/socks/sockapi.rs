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
// use std::thread::{Builder, JoinHandle};

use crate::{
    socks::{
        socks::*,
        message::UdpPayload,
    },
};
use std::{
    time::Instant,
    fmt::Debug
};

#[macro_export]
macro_rules! sync {
    ($name:expr, $target_names:expr, $task_name:expr, $default_context:expr, |$context:ident: $U:ty, $($target:ident: $T:ty),+| $body:expr) => (
        Sock::synced($name, $target_names, $task_name, $default_context,  build_fn!(|$context: $U, $($target: $T),+| $body))
    );
    ($name:expr, $target_names:expr, $task_name:expr, $default_context:expr,  |$context:ident: $U:ty, $t:ident, $($target:ident: $T:ty),+| $body:expr) => (
        Sock::synced($name, $target_names, $task_name, $default_context, build_fn!(|$context: $U, $t, $($target: $T),+| $body))
    );
}

#[macro_export]
macro_rules! unsync {
    ($name:expr, $target_names:expr, $task_name:expr, $default_context:expr, |$context:ident: $U:ty, $($target:ident: $T:ty),+| $body:expr) => (
        Sock::unsynced($name, $target_names, $task_name, $default_context,  build_fn!(|$context: $U, $($target: $T),+| $body))
    );
    ($name:expr, $target_names:expr, $task_name:expr, $default_context:expr,  |$context:ident: $U:ty, $t:ident, $($target:ident: $T:ty),+| $body:expr) => (
        Sock::unsynced($name, $target_names, $task_name, $default_context, build_fn!(|$context: $U, $t, $($target: $T),+| $body))
    );
}

#[macro_export]
macro_rules! add_task {
    ($sock:expr, $target_name:expr, $task_name:expr, $default_context:expr,  |$context:ident: $U:ty, $($target:ident: $T:ty),+| $body:expr) => (
        $sock.link_task($task_name, $target_name, $default_context, build_fn!(|$context: $U, $($target: $T),+| $body));
    );

    ($sock:expr, $target_name:expr, $task_name:expr, $default_context:expr,  |$context:ident: $U:ty, $t:ident, $($target:ident: $T:ty),+| $body:expr) => (
        $sock.link_task($task_name, $target_name, $default_context, build_fn!(|$context: $U, $t, $($target: $T),+| $body));
    );
}


///
///
///
///     Nice for figuring things out
///
///
///
///

pub fn shutdown() { // what the fuck is this?
    let mut sock = Sock::source("shutdown");
    sock.tx_payload(0);
    sock.tx_payload(0);
    sock.tx_payload(0);
    sock.tx_payload(0);
    let t = Instant::now();
    while t.elapsed().as_secs() < 1 {}
    sock.tx_payload(0);
    sock.tx_payload(0);
    sock.tx_payload(0);
    sock.tx_payload(0);
}

pub fn sync_echo<T: PartialEq + Debug + for<'a> serde::de::Deserialize<'a>>(name: &str, targets: Vec<&str>) {

    let mut sock = Sock::synced("echo", targets, name, 0, |data: Vec<UdpPayload>, _ctx: &mut UdpPayload, t: f64| {
        let payloads: Vec<T> = data.iter().map(|task_in| bincode::deserialize::<T>(&task_in).expect("Failed to deserialize input")).collect();
        println!("[{t:.6}] {payloads:?}");
        (0, vec![])
    });

    sock.spin();
}

pub fn echo<T: PartialEq + Debug + for<'a> serde::de::Deserialize<'a>>(name: &str, targets: Vec<&str>) {

    let mut sock = Sock::unsynced("echo", targets, name, 0, |data: Vec<UdpPayload>, _ctx: &mut UdpPayload, t: f64| {
        let payloads: Vec<T> = data.iter().map(|task_in| bincode::deserialize::<T>(&task_in).expect("Failed to deserialize input")).collect();
        println!("[{t:.6}] {payloads:?}");
        (0, vec![])
    });

    sock.spin();
}

pub fn hz<T: PartialEq + Debug + for<'a> serde::de::Deserialize<'a>>(name: &str, targets: Vec<&str>) {
    
    let mut sock = Sock::synced("hz", targets, name, 0.0f64, |_data: Vec<UdpPayload>, ctx: &mut UdpPayload, t: f64| {
        let t1: f64 = bincode::deserialize(ctx).unwrap();
        *ctx = bincode::serialize(&t).unwrap();
        println!("[{t:.6}] {:.4}", 1.0 / (t - t1));
        (0, vec![])
    });

    sock.spin();
}
