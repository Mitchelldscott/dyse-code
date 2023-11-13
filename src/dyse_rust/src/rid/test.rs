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
    rid::{data_structures::*, interface::*, layer::*, reader::*, robot_firmware::*, writer::*},
    socks::sockapi,
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
pub static TEST_DURATION: u64 = 30;

#[cfg(test)]
pub mod robot_fw {
    use super::*;

    #[test]
    pub fn robot_fw_load() {
        let rs = RobotFirmware::new("penguin");

        rs.print();

        rs.all_init_packets().iter().for_each(|packet| {
            let mut total_items = 0;
            // packet.print();
            packet.iter().for_each(|b| {
                if *b != 0 {
                    total_items += 1;
                }
            });

            match total_items < 16 {
                true => {
                    println!("almost empty packet {total_items} {packet:?}");
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
                let mut buffer = [0; HID_PACKET_SIZE];
                buffer[HID_MODE_INDEX] = 255;
                buffer[HID_TOGL_INDEX] = 255;
                interface
                    .layer
                    .pc_stats
                    .n_tx()
                    .to_le_bytes()
                    .iter()
                    .enumerate()
                    .for_each(|(i, &b)| buffer[HID_TASK_INDEX + i] = b);
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
            (interface.layer.pc_stats.n_tx() - interface.layer.mcu_stats.n_tx()).abs(),
            (TEST_DURATION * 5) as f64,
            "PC and MCU sent different numbers of packets"
        );
        assert_le!(
            ((TEST_DURATION as f64 / TEENSY_CYCLE_TIME_S) - interface.layer.mcu_stats.n_tx()).abs(),
            (TEST_DURATION * 500) as f64,
            "Not enough packts sent by mcu"
        );
        assert_le!(
            ((TEST_DURATION as f64 / TEENSY_CYCLE_TIME_S) - interface.layer.pc_stats.n_tx()).abs(),
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
        while !interface.layer.control_flags.is_connected() {}

        let lifetime = Instant::now();
        let mut t = Instant::now();

        println!("[HID-Control]: Live");

        while lifetime.elapsed().as_secs() < TEST_DURATION
            && !interface.layer.control_flags.is_shutdown()
        {
            let loopt = Instant::now();

            if !interface.layer.control_flags.is_connected()
                || !interface.layer.control_flags.is_initialized()
            {
                interface.robot_fw.configured = vec![false; interface.robot_fw.tasks.len()];
                interface.layer.control_flags.initialize(true);
                interface.send_initializers();
            } else {
                interface.try_config();

                match interface.robot_fw.parse_sock() {
                    Some(packet) => interface.writer_tx(packet),
                    _ => {}
                }

                interface.check_feedback();

                if t.elapsed().as_secs() >= 20 {
                    interface.print();
                    t = Instant::now();
                }
            }

            interface.layer.delay(loopt);
        }

        interface.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        interface.print();

        let status: Vec<String> = (0..interface.robot_fw.tasks.len())
            .filter_map(|i| match interface.robot_fw.configured[i] {
                true => None,
                false => Some(interface.robot_fw.tasks[i].name.clone()),
            })
            .collect();

        let timestamps: Vec<String> = (0..interface.robot_fw.tasks.len())
            .filter_map(|i| match interface.robot_fw.tasks[i].pc_time != 0.0 {
                true => None,
                false => Some(interface.robot_fw.tasks[i].name.clone()),
            })
            .collect();

        sockapi::shutdown();
        assert_eq!(0, status.len(), "Failed to configure {:?}", status);
        assert_eq!(
            0,
            timestamps.len(),
            "Didn't receive output from {:?}",
            timestamps
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

        reader_handle.join().expect("[HID-Reader]: failed");
        interface_sim.join().expect("[HID-Control]: failed");
        writer_handle.join().expect("[HID-Writer]: failed");
    }

    // #[test]
    // pub fn demo_echo() {
    //     sockapi::sync_echo::<TaskCommunication>("echo", vec!["signal"]);
    // }

    // #[test]
    // pub fn demo_hz() {
    //     sockapi::hz::<TaskCommunication>("hz", vec!["lsm9ds1"]);
    // }
}
