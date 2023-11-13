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

use crate::{
    rid::{data_structures::*, layer::*, reader::*, robot_firmware::*, writer::*},
    socks::sockapi,
};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Instant,
};

use chrono::{DateTime, Utc};

pub static MCU_NO_COMMS_TIMEOUT_S: u64 = 10;
pub static MCU_NO_COMMS_RESET_MS: u128 = 10;
pub static MCU_RECONNECT_DELAY_US: f64 = 5.0 * 1E6;

pub static TEENSY_CYCLE_TIME_S: f64 = 0.001;
pub static TEENSY_CYCLE_TIME_MS: f64 = TEENSY_CYCLE_TIME_S * 1E3;
pub static TEENSY_CYCLE_TIME_US: f64 = TEENSY_CYCLE_TIME_S * 1E6;
pub static TEENSY_CYCLE_TIME_ER: f64 = TEENSY_CYCLE_TIME_US + 50.0; // err threshold (before prints happen, deprecated?)

pub static TEENSY_DEFAULT_VID: u16 = 0x16C0;
pub static TEENSY_DEFAULT_PID: u16 = 0x0486;

pub struct HidInterface {
    pub layer: HidLayer,

    // For sending reports to the writer
    pub reader_rx: Receiver<(HidPacket, DateTime<Utc>)>,
    pub writer_tx: Sender<HidPacket>,

    // For storing reply data
    pub robot_fw: RobotFirmware,
}

impl HidInterface {
    pub fn new() -> (HidInterface, HidReader, HidWriter) {
        let layer = HidLayer::new(TEENSY_DEFAULT_VID, TEENSY_DEFAULT_PID, TEENSY_CYCLE_TIME_US);
        let (writer_tx, writer_rx) = channel::<HidPacket>();
        let (reader_tx, reader_rx) = channel::<(HidPacket, DateTime<Utc>)>();

        (
            HidInterface {
                layer: layer.clone(),
                reader_rx: reader_rx,
                writer_tx: writer_tx,
                robot_fw: RobotFirmware::default(),
            },
            HidReader::new(layer.clone(), reader_tx),
            HidWriter::new(layer, writer_rx),
        )
    }

    pub fn sim() -> HidInterface {
        let (hidui, _, _) = HidInterface::new();
        hidui
    }

    pub fn writer_tx(&self, buffer: HidPacket) {
        match self.writer_tx.send(buffer) {
            Ok(_) => {}
            _ => self.layer.control_flags.shutdown(),
        };
    }

    pub fn check_feedback(&mut self) {
        match self.reader_rx.try_recv() {
            Ok((buffer, datetime)) => {
                let run_time = f32::from_le_bytes(
                    buffer[HID_RUNT_INDEX..HID_RUNT_INDEX + 4]
                        .try_into()
                        .unwrap(),
                ) as f64;

                let replied_mcu_time = f32::from_le_bytes(
                    buffer[HID_RUCT_INDEX..HID_RUCT_INDEX + 4]
                        .try_into()
                        .unwrap(),
                ) as f64;

                let replied_pc_time = f32::from_le_bytes(
                    buffer[HID_PCTS_INDEX..HID_PCTS_INDEX + 4]
                        .try_into()
                        .unwrap(),
                ) as f64;

                let mcu_time = self
                    .layer
                    .mcu_stats
                    .from_bytes(&buffer[HID_UCTS_INDEX..HID_UCTS_INDEX + 4]);
                let pc_time = self.layer.pc_stats.from_utcs(datetime, self.layer.datetime) as f64;

                match (buffer[0], pc_time - replied_pc_time < 0.05) {
                    (255, true) => {
                        let packets_tx = f32::from_le_bytes(
                            buffer[HID_TASK_INDEX..HID_TASK_INDEX + 4]
                                .try_into()
                                .unwrap(),
                        ) as f64;
                        let packets_rx = f32::from_le_bytes(
                            buffer[HID_TASK_INDEX + 4..HID_TASK_INDEX + 8]
                                .try_into()
                                .unwrap(),
                        ) as f64;

                        // println!("[HID-Control]: Packet Sync\t{}\t{}", packets_tx - self.layer.mcu_stats.n_tx(), packets_rx - self.layer.mcu_stats.n_rx());

                        self.layer.mcu_stats.set_tx(packets_tx);
                        self.layer.mcu_stats.set_rx(packets_rx);
                    }
                    (1, true) => {
                        self.robot_fw.parse_hid(
                            pc_time,
                            mcu_time,
                            run_time,
                            buffer[HID_TASK_INDEX] as usize,
                            buffer,
                        );

                        self.layer.mcu_stats.update_tx(1.0); // only works if we don't miss packets
                        self.layer.mcu_stats.update_rx(1.0);
                    }
                    (_, true) => println!("[HID-Control]: Unknown Report Mode\t{buffer:?}"),
                    (_, false) => println!(
                        "[HID-Control]: Large Time drift\t{:.6} {:.6}",
                        pc_time - replied_pc_time,
                        mcu_time - replied_mcu_time,
                    ),
                };
            }
            _ => {}
        }
    }

    pub fn read_write_spin(&mut self, packets: Vec<HidPacket>) {
        packets.into_iter().for_each(|packet| {
            let t = Instant::now();
            self.writer_tx(packet);
            self.check_feedback();
            self.layer.delay(t);
        });
    }

    pub fn send_initializers(&mut self) {
        self.read_write_spin(self.robot_fw.all_init_packets());
    }

    pub fn try_config(&mut self) {
        self.read_write_spin(self.robot_fw.unconfigured_parameters());
    }

    pub fn pipeline(&mut self, _unused_flag: bool) {
        while !self.layer.control_flags.is_connected() {}

        let mut t = Instant::now();

        println!("[HID-Control]: Live");

        while !self.layer.control_flags.is_shutdown() {
            let loopt = Instant::now();

            if !self.layer.control_flags.is_connected()
                || !self.layer.control_flags.is_initialized()
            {
                self.robot_fw.configured = vec![false; self.robot_fw.tasks.len()];
                self.layer.control_flags.initialize(true);
                self.send_initializers();
            } else {
                self.try_config();

                match self.robot_fw.parse_sock() {
                    Some(packet) => self.writer_tx(packet),
                    _ => {}
                }

                self.check_feedback();

                if t.elapsed().as_secs() >= 20 {
                    self.print();
                    t = Instant::now();
                }
            }

            // self.layer.delay(loopt);
            if self.layer.delay(loopt) > TEENSY_CYCLE_TIME_US {
                println!(
                    "[HID-Control]: over cycled {:.6}s",
                    1E-6 * (t.elapsed().as_micros() as f64)
                );
            }
        }

        sockapi::shutdown();
        self.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        self.layer.print();
    }

    pub fn print(&self) {
        self.layer.print();
        self.robot_fw.print();
    }
}
