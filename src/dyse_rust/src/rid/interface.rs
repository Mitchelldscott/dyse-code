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
    rid::{
        layer::*, 
        reader::*, 
        writer::*, 
        robot_firmware::*,
        data_structures::*,
    },
};
use std::{sync::mpsc::{channel, Sender, Receiver}, time::Instant};

pub static MCU_NO_COMMS_TIMEOUT_S: u64 = 10;
pub static MCU_NO_COMMS_RESET_MS: u128 = 10;
pub static MCU_RECONNECT_DELAY_US: f64 = 5.0 * 1E6;

pub static TEENSY_CYCLE_TIME_S: f64 = 0.0002;
pub static TEENSY_CYCLE_TIME_MS: f64 = TEENSY_CYCLE_TIME_S * 1E3;
pub static TEENSY_CYCLE_TIME_US: f64 = TEENSY_CYCLE_TIME_S * 1E6;
pub static TEENSY_CYCLE_TIME_ER: f64 = TEENSY_CYCLE_TIME_US + 50.0; // err threshold (before prints happen)

pub static TEENSY_DEFAULT_VID: u16 = 0x16C0;
pub static TEENSY_DEFAULT_PID: u16 = 0x0486;

pub fn get_latch_packet(i: u8, latch: u8, data: &Vec<f64>) -> HidPacket {
    let mut buffer = [0; HID_PACKET_SIZE];
    buffer[HID_MODE_INDEX] = TASK_CONTROL_ID;
    buffer[HID_TOGL_INDEX] = latch;
    buffer[HID_TASK_INDEX] = i;
    buffer[HID_DATA_INDEX] = data.len() as u8;
    data.iter().enumerate().for_each(|(i, &x)| (x as f32).to_le_bytes().iter().enumerate().for_each(|(j, &b)| buffer[(4*i)+j] = b));
    buffer
}

pub fn disable_latch(i: u8) -> HidPacket {
    get_latch_packet(i, 0, &vec![])
}

pub fn output_latch(i: u8, data: &Vec<f64>) -> HidPacket {
    get_latch_packet(i, 1, data)
}

pub fn input_latch(i: u8, data: &Vec<f64>) -> HidPacket {
    get_latch_packet(i, 2, data)
}

pub struct HidInterface {
    pub layer: HidLayer,

    // For sending reports to the writer
    pub reader_rx: Receiver<HidPacket>,
    pub writer_tx: Sender<HidPacket>,

    // For storing reply data
    pub robot_fw: RobotFirmware,
}

impl HidInterface {
    pub fn new() -> (HidInterface, HidReader, HidWriter) {
        let layer = HidLayer::new(TEENSY_DEFAULT_VID, TEENSY_DEFAULT_PID, TEENSY_CYCLE_TIME_US);
        let (writer_tx, writer_rx) = channel::<HidPacket>();
        let (reader_tx, reader_rx) = channel::<HidPacket>();

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
            Ok(_) => {},
            _ => self.layer.control_flags.shutdown(),
        };
    }

    pub fn check_feedback(&mut self) {
        match self.reader_rx.try_recv() {
            Ok(report) => {
                self.robot_fw
                    .parse_hid_feedback(report, &self.layer.mcu_stats);
            }
            _ => {}
        }
    }

    pub fn send_initializers(&self) {
        self.robot_fw.task_init_packets().into_iter().for_each(|packet| {
            let t = Instant::now();
            self.writer_tx(packet);
            self.layer.delay(t);
        });
    }

    pub fn try_config(&self) {
        self.robot_fw
            .task_param_packets()
            .into_iter()
            .for_each(|packet| {
                println!(
                    "sending config for {} {}",
                    packet[2],
                    self.layer.pc_stats.lifetime()
                );
                let t = Instant::now();
                self.writer_tx(packet);
                self.layer.delay(t);
            });
    }

    pub fn send_olatch(&mut self, i: usize, data: Vec<f64>) {
        self.writer_tx(output_latch(i as u8, &data));
    }

    pub fn send_ilatch(&mut self, i: usize, data: Vec<f64>) {
        self.writer_tx(input_latch(i as u8, &data));
    }

    pub fn print(&self) {
        self.layer.print();
        self.robot_fw.print();
    }

    pub fn pipeline(&mut self, _unused_flag: bool) {
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
        }

        self.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        self.layer.print();
    }
}
