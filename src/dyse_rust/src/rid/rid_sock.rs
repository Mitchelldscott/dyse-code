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

use dyse_rust::{
    socks::socks::*,
    rid::{hid_interface::*},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::Builder,
};

fn main() {
    /*
        Start an hid layer
    */
    let core_handle = Builder::new()
        .name("Sock Core".to_string())
        .spawn(move || {
            let mut sock = Sockage::core("sock-core");

            assert_eq!(
                sock.socket.local_addr().unwrap(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
                "Core Didn't bind to requested IP"
            );

            while !sock.is_shutdown() {
                sock.core_parse();
            }

            sock.send_terminate();
            sock.log_heavy();
        })
        .unwrap();

    let (mut interface, mut reader, mut writer) = HidInterface::new();

    interface.print();

    let reader_handle = Builder::new()
        .name("HID Reader".to_string())
        .spawn(move || {
            reader.pipeline();
        })
        .unwrap();

    let writer_handle = Builder::new()
        .name("HID Writer".to_string())
        .spawn(move || {
            writer.pipeline();
        })
        .unwrap();

    let interface_handle = Builder::new()
        .name("HID Control".to_string())
        .spawn(move || {
            interface.pipeline(true);
        })
        .unwrap();

    interface_handle.join().expect("HID Control failed");
    writer_handle.join().expect("HID Writer failed");
    reader_handle.join().expect("HID Reader failed");

    Sockage::shutdown_socks();
    core_handle.join().expect("Sock core failed");
}
