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
use hidapi::HidDevice;

use crate::{
    rid::{
        layer::*,
        data_structures::{
            HidPacket,
            HID_PACKET_SIZE,
        },
    },
};

use std::{
    sync::mpsc::Sender, 
    time::Instant
};

/// Reads from an Hid Device and send the packets through a channel
pub struct HidReader {
    parser_tx: Sender<HidPacket>,
    teensy: HidDevice,
    layer: HidLayer,
    timestamp: Instant,
}

impl HidReader {
    pub fn new(layer: HidLayer, parser_tx: Sender<HidPacket>) -> HidReader {
        HidReader {
            parser_tx: parser_tx,
            teensy: layer.wait_for_device(),
            layer: layer,
            timestamp: Instant::now(),
        }
    }

    pub fn print(&self) {
        println!(
            "Reader Dump\n\trust time: {}\n\tteensy time: {}",
            self.layer.pc_stats.lifetime(),
            self.layer.mcu_stats.lifetime(),
        );
    }

    pub fn reconnect(&mut self) {
        // check reconnect after 1000 cycles
        if self.timestamp.elapsed().as_millis() as f64 > self.layer.sample_time {
            if self.layer.control_flags.is_connected() {
                println!(
                    "[HID-Reader]: hasn't written for {}s",
                    (self.timestamp.elapsed().as_millis() as f64) * 1E-3
                );
            }

            self.teensy = self.layer.wait_for_device();
        }
    }

    /// Read data into the input buffer and return how many bytes were read
    ///
    /// # Usage
    ///
    /// ```
    /// match reader.read() {
    ///     64 => {
    ///         // packet OK, do something
    ///     }
    ///     _ => {} // do nothing
    /// }
    /// ```
    pub fn read(&mut self) -> usize {
        let mut buffer = [0; HID_PACKET_SIZE];
        match &self.teensy.read(&mut buffer) {
            Ok(value) => {
                if *value == HID_PACKET_SIZE {

                    match self.parser_tx.send(buffer) {
                        Ok(_) => {},
                        _ => self.layer.control_flags.shutdown(),
                    };

                    self.timestamp = Instant::now();
                    self.layer.pc_stats.update_packets_read(1.0);
                }

                *value
            }
            _ => {
                self.layer.control_flags.initialize(false);
                self.reconnect();
                0
            }
        }
    }

    /// Main function to spin and connect the teensys
    /// input to Socks.
    ///
    /// # Usage
    /// ```
    /// ```
    pub fn spin(&mut self) {

        while !self.layer.control_flags.is_shutdown() {
            
            let loopt = Instant::now();

            self.read();

            self.layer.delay(loopt);
        }

    }

    pub fn pipeline(&mut self) {
        println!("[HID-reader]: Live");

        self.spin();

        println!("[HID-reader]: Shutdown");
    }
}
