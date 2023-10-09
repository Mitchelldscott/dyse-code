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

extern crate hidapi;

use crate::{
    comms::{robot_firmware::*, hid_layer::*, hid_reader::*, hid_writer::*},
    utilities::data_structures::*,
};
use std::{sync::mpsc, time::Instant};

pub static MCU_NO_COMMS_TIMEOUT_S: u64 = 10;
pub static MCU_NO_COMMS_RESET_MS: u128 = 10;
pub static MCU_RECONNECT_DELAY_US: f64 = 5.0 * 1E6;

pub static TEENSY_CYCLE_TIME_S: f64 = 0.0002;
pub static TEENSY_CYCLE_TIME_MS: f64 = TEENSY_CYCLE_TIME_S * 1E3;
pub static TEENSY_CYCLE_TIME_US: f64 = TEENSY_CYCLE_TIME_S * 1E6;
pub static TEENSY_CYCLE_TIME_ER: f64 = TEENSY_CYCLE_TIME_US + 50.0; // err threshold (before prints happen)

pub static TEENSY_DEFAULT_VID: u16 = 0x16C0;
pub static TEENSY_DEFAULT_PID: u16 = 0x0486;

pub struct HidInterface {
    pub layer: HidLayer,

    pub current_request: u8,

    // For sending reports to the writer
    pub writer_tx: crossbeam_channel::Sender<ByteBuffer>,
    pub parser_rx: mpsc::Receiver<ByteBuffer>,

    // For storing reply data
    pub robot_fw: RobotFirmware,
}

impl HidInterface {
    pub fn new() -> (HidInterface, HidReader, HidWriter) {
        let layer = HidLayer::new(TEENSY_DEFAULT_VID, TEENSY_DEFAULT_PID, TEENSY_CYCLE_TIME_US);
        let (writer_tx, writer_rx): (
            crossbeam_channel::Sender<ByteBuffer>,
            crossbeam_channel::Receiver<ByteBuffer>,
        ) = crossbeam_channel::bounded(100);
        let (parser_tx, parser_rx): (mpsc::Sender<ByteBuffer>, mpsc::Receiver<ByteBuffer>) =
            mpsc::channel();

        (
            HidInterface {
                layer: layer.clone(),

                current_request: 0,

                writer_tx: writer_tx.clone(),
                parser_rx: parser_rx,

                robot_fw: RobotFirmware::default(writer_tx),
            },
            HidReader::new(layer.clone(), parser_tx),
            HidWriter::new(layer, writer_rx),
        )
    }

    pub fn sim() -> HidInterface {
        let layer = HidLayer::new(TEENSY_DEFAULT_VID, TEENSY_DEFAULT_PID, TEENSY_CYCLE_TIME_US);
        let (writer_tx, _): (
            crossbeam_channel::Sender<ByteBuffer>,
            crossbeam_channel::Receiver<ByteBuffer>,
        ) = crossbeam_channel::bounded(100);
        let (_, parser_rx): (mpsc::Sender<ByteBuffer>, mpsc::Receiver<ByteBuffer>) =
            mpsc::channel();

        HidInterface {
            layer: layer.clone(),

            current_request: 0,

            writer_tx: writer_tx.clone(),
            parser_rx: parser_rx,

            robot_fw: RobotFirmware::default(writer_tx),
        }
    }

    pub fn writer_tx(&self, report: ByteBuffer) {
        self.writer_tx.send(report).unwrap();
    }

    pub fn check_feedback(&mut self) {
        match self.parser_rx.try_recv() {
            Ok(report) => {
                self.robot_fw
                    .parse_hid_feedback(report, &self.layer.mcu_stats);
            }
            _ => {}
        }
    }

    pub fn send_initializers(&self) {
        self.robot_fw.task_init_packets().iter().for_each(|packet| {
            let t = Instant::now();
            self.writer_tx(packet.clone());
            self.layer.delay(t);
        });
    }

    pub fn try_config(&self) {
        self.robot_fw
            .task_param_packets()
            .iter()
            .for_each(|packet| {
                println!(
                    "sending config for {} {}",
                    packet.get(2),
                    self.layer.pc_stats.lifetime()
                );
                let t = Instant::now();
                self.writer_tx(packet.clone());
                self.layer.delay(t);
            });
    }

    pub fn send_olatch(&mut self, i: usize, data: Vec<f64>) {
        self.writer_tx(output_latch(i as u8, data));
    }

    pub fn send_ilatch(&mut self, i: usize, data: Vec<f64>) {
        self.writer_tx(input_latch(i as u8, data));
    }

    pub fn print(&self) {
        self.layer.print();
        self.robot_fw.print();
    }

    pub fn pipeline(&mut self, _unused_flag: bool) {
        self.send_initializers();
        while !self.layer.control_flags.is_connected() {}

        println!("[HID-Control]: Live");

        let mut t = Instant::now();

        while !self.layer.control_flags.is_shutdown() {
            let loopt = Instant::now();

            if !self.layer.control_flags.is_connected()
                || !self.layer.control_flags.is_initialized()
            {
                self.robot_fw.configured = vec![false; self.robot_fw.tasks.len()];
                self.layer.control_flags.initialize(true);
                self.send_initializers();
            } else {
                self.check_feedback();

                if t.elapsed().as_secs() >= 20 {
                    self.print();
                    t = Instant::now();
                }
            }

            self.layer.loop_delay(loopt);
            // if t.elapsed().as_micros() as f64 > TEENSY_CYCLE_TIME_US {
            //     println!("HID Control over cycled {} ms", (t.elapsed().as_micros() as f64) * 1E-3);
            // }
        }

        self.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        self.layer.print();
    }
}
