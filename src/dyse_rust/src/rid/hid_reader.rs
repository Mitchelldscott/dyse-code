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
    rid::hid_layer::*,
    utilities::data_structures::*,
};

use hidapi::HidDevice;
use std::{sync::mpsc::Sender, time::Instant};

/// Responsible for initializing [RobotStatus] and continuously
/// sending status reports
pub struct HidReader {
    parser_tx: Sender<ByteBuffer>,
    input: ByteBuffer,
    teensy: HidDevice,
    layer: HidLayer,
    sock: Sock,
    timestamp: Instant,
}

impl HidReader {
    pub fn new(layer: HidLayer, parser_tx: Sender<ByteBuffer>) -> HidReader {
        HidReader {
            parser_tx: parser_tx,
            input: ByteBuffer::hid(),
            teensy: layer.wait_for_device(),
            layer: layer,
            sock: Sock::source("rid/rx")
            timestamp: Instant::now(),
        }
    }

    pub fn print(&self) {
        println!(
            "Reader Dump\n\trust time: {}\n\tteensy time: {}",
            self.layer.pc_stats.lifetime(),
            self.layer.mcu_stats.lifetime(),
        );
        self.input.print();
    }

    pub fn buffer(&self) -> ByteBuffer {
        self.input.clone()
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
        // let t = Instant::now();
        let mut buffer = [0; 64];
        match &self.teensy.read(&mut buffer) {
            Ok(value) => {
                if *value == 64 {
                    self.layer.pc_stats.update_packets_read(1.0);
                    self.timestamp = Instant::now();
                    self.sock.tx_payload(buffer);
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

    /// After requesting a report this can be used to wait for a reply
    ///
    ///
    /// # Usage
    ///
    /// ```
    /// // set writer.output.data[0] to an int [1-3] (255 for initializer)
    /// writer.write();
    /// reader.wait_for_reply(255, 10);
    /// ```
    pub fn wait_for_reply(&mut self, packet_id: u8, timeout: u128) {
        let wait_timer = Instant::now();

        while wait_timer.elapsed().as_millis() < timeout {
            match self.read() {
                64 => {
                    if self.input.get(0) == packet_id {
                        self.layer.control_flags.initialize(true);
                        return;
                    }
                }
                _ => {}
            }

        }

        // If packet never arrives
        self.layer.control_flags.shutdown();
        println!("HID Reader timed out waiting for reply from Teensy");
    }

    /// Main function to spin and connect the teensys
    /// input to Socks.
    ///
    /// # Usage
    /// ```
    /// use hidapi::HidApi;
    /// use dyse_rust::rid::HidReader;
    ///
    /// let mut hidapi = HidApi::new().expect("Failed to create API instance");
    /// let mut reader = HidReader::new(&mut hidapi, vid, pid);
    /// reader.spin();       // runs until watchdog times out
    /// ```
    pub fn spin(&mut self) {
        self.wait_for_reply(255, 100);

        while !self.layer.control_flags.is_shutdown() {
            let loopt = Instant::now();

            self.read();

            self.layer.delay(loopt);
        }

        self.wait_for_reply(255, 100);
    }

    /// Sends robot status report packet to [HidROS], waits for the reply packet,
    /// then calls [HidReader::spin] to begin parsing reports
    ///
    /// # Example
    ///
    /// see [HidLayer::pipeline()]
    pub fn pipeline(&mut self) {
        println!("[HID-reader]: Live");

        self.spin();

        println!("[HID-reader]: Shutdown");
    }
}
