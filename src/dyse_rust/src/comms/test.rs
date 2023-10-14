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
extern crate hidapi;

use hidapi::{HidApi, HidDevice};

use crate::{
    comms::{
        data_structures::*, hid_interface::*, hid_layer::*, hid_reader::*, hid_writer::*,
        robot_firmware::*, socks::*,
    },
    utilities::{data_structures::*, loaders::*},
};

use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
    thread::{spawn, Builder},
    time::{Duration, Instant},
};

use more_asserts::assert_le;

#[allow(dead_code)]
const VERBOSITY: usize = 1;
pub static TEST_DURATION: u64 = 10;

#[cfg(test)]
pub mod robot_fw {
    use super::*;

    #[test]
    pub fn robot_fw_load() {
        let (writer_tx, _): (
            crossbeam_channel::Sender<ByteBuffer>,
            crossbeam_channel::Receiver<ByteBuffer>,
        ) = crossbeam_channel::bounded(100);

        let rs = RobotFirmware::new("penguin", writer_tx);

        rs.print();

        rs.task_init_packets().iter().for_each(|packet| {
            let mut total_items = 0;
            // packet.print();
            packet.buffer().iter().for_each(|b| {
                if *b != 0 {
                    total_items += 1;
                }
            });

            match total_items < 16 {
                true => {
                    println!("almost empty packet {total_items}");
                    packet.print();
                }
                false => {}
            };
        });
    }
}

///
/// Test the hid functionality on the Teensy
/// Only demonstrates the ability to maintain a connection
/// This is usually paried with firmware/examples/hid/live_test.cpp
/// Dump packets to the Teensy at 1ms, each packet contains a counter
/// and timestamp.
#[cfg(test)]
pub mod dead_read_write {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    pub fn hid_read_write_spawner() {
        /*
            Start an hid layer
        */
        // let (layer, writer_rx) = HidLayer::new("penguin");
        // let sim_layer = layer.clone();
        // let mut hidreader = HidReader::new(layer.clone());
        // let mut hidwriter = HidWriter::new(layer, writer_rx);

        let (mut interface, mut reader, mut writer) = HidInterface::new();

        interface.layer.print();

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

        let t = Instant::now();
        while t.elapsed().as_secs() < TEST_DURATION && !interface.layer.control_flags.is_shutdown()
        {
            let loopt = Instant::now();
            interface.check_feedback();
            interface.layer.delay(loopt);
        }
        interface.layer.control_flags.shutdown();

        reader_handle.join().expect("[HID-Reader]: failed");
        writer_handle.join().expect("[HID-Writer]: failed");
        interface.layer.print();
    }
}

#[cfg(test)]
pub mod dead_comms {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    pub fn sim_interface(mut interface: HidInterface) {
        while !interface.layer.control_flags.is_connected() {}

        println!("[HID-Control]: Live");

        let t = Instant::now();

        while t.elapsed().as_secs() < TEST_DURATION && !interface.layer.control_flags.is_shutdown()
        {
            let loopt = Instant::now();

            if interface.layer.control_flags.is_connected() {
                let mut buffer = ByteBuffer::hid();
                buffer.puts(0, vec![255, 255]);
                buffer.put_float(2, interface.layer.pc_stats.packets_sent());
                interface.writer_tx(buffer);
                interface.check_feedback();
            }

            interface.layer.delay(loopt);
            // if interface.delay(t) > TEENSY_CYCLE_TIME_US {
            //     println!("HID Control over cycled {}", t.elapsed().as_micros());
            // }
        }

        interface.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        interface.print();

        assert_le!(
            (interface.layer.pc_stats.packets_sent() - interface.layer.mcu_stats.packets_sent())
                .abs(),
            (TEST_DURATION * 5) as f64,
            "PC and MCU sent different numbers of packets"
        );
        assert_le!(
            ((TEST_DURATION as f64 / TEENSY_CYCLE_TIME_S)
                - interface.layer.mcu_stats.packets_sent())
            .abs(),
            (TEST_DURATION * 500) as f64,
            "Not enough packts sent by mcu"
        );
        assert_le!(
            ((TEST_DURATION as f64 / TEENSY_CYCLE_TIME_S)
                - interface.layer.pc_stats.packets_sent())
            .abs(),
            (TEST_DURATION * 500) as f64,
            "Not enough packts sent by pc"
        );
    }

    #[test]
    pub fn hid_spawner() {
        /*
            Start an hid layer
        */
        let (interface, mut reader, mut writer) = HidInterface::new();

        interface.layer.print();

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

        let interface_sim = Builder::new()
            .name("HID Control".to_string())
            .spawn(move || {
                sim_interface(interface);
            })
            .unwrap();

        reader_handle.join().expect("HID Reader failed");
        interface_sim.join().expect("HID Control failed");
        writer_handle.join().expect("HID Writer failed");
    }
}

#[cfg(test)]
pub mod live_comms {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    pub fn sim_interface(mut interface: HidInterface) {
        interface.send_initializers();

        while !interface.layer.control_flags.is_connected() {}

        println!("[HID-Control]: Live");

        let lifetime = Instant::now();
        let mut t = Instant::now();

        while lifetime.elapsed().as_secs() < TEST_DURATION
            && !interface.layer.control_flags.is_shutdown()
        {
            let loopt = Instant::now();
            // interface.try_config();

            if interface.layer.control_flags.is_connected() {
                interface.check_feedback();
                if t.elapsed().as_secs() >= 10 {
                    interface.print();
                    t = Instant::now();
                }
            }

            interface.layer.loop_delay(loopt);
            // if t.elapsed().as_micros() as f64 > TEENSY_CYCLE_TIME_US {
            //     println!("HID Control over cycled {} ms", (t.elapsed().as_micros() as f64) * 1E-3);
            // }
        }

        interface.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        interface.layer.print();

        let mut status = vec![false; interface.robot_fw.tasks.len()];
        (0..interface.robot_fw.tasks.len()).for_each(|i| {
            match interface.robot_fw.configured[i] {
                true => {}
                false => {
                    status[i] = true;
                }
            };
        });

        status
            .iter()
            .enumerate()
            .for_each(|(i, status)| assert_eq!(*status, false, "Failed to configure task {}", i));
    }

    #[test]
    pub fn hid_spawner() {
        /*
            Start an hid layer
        */

        let (interface, mut reader, mut writer) = HidInterface::new();

        interface.layer.print();

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

        let interface_sim = Builder::new()
            .name("HID Control".to_string())
            .spawn(move || {
                sim_interface(interface);
            })
            .unwrap();

        reader_handle.join().expect("[HID-Reader]: failed");
        interface_sim.join().expect("[HID-Control]: failed");
        writer_handle.join().expect("[HID-Writer]: failed");
    }

    #[test]
    pub fn core() {
        let mut sock = Sockage::core("live_comms_core");

        assert_eq!(
            sock.socket.local_addr().unwrap(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1313),
            "Core Didn't bind to requested IP"
        );

        while sock.lifetime.elapsed().as_secs() < TEST_DURATION {
            sock.core_parse();
        }

        sock.send_terminate();
        sock.log_heavy();
    }

    #[test]
    pub fn echo_node() {
        Sockage::echo(vec!["signal".to_string()]);
    }
}
