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

use crate::rid::hid_layer::*;
use crate::utilities::data_structures::*;

use hidapi::HidDevice;
use std::{sync::mpsc::Sender, time::Instant};

/// Responsible for initializing [RobotStatus] and continuously
/// sending status reports
pub struct HidReader {
    parser_tx: Sender<ByteBuffer>,
    input: ByteBuffer,
    teensy: HidDevice,
    layer: HidLayer,
    timestamp: Instant,
}

impl HidReader {
    pub fn new(layer: HidLayer, parser_tx: Sender<ByteBuffer>) -> HidReader {
        HidReader {
            parser_tx: parser_tx,
            input: ByteBuffer::hid(),
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
        match &self.teensy.read(&mut self.input.data) {
            Ok(value) => {
                // reset watchdog... woof
                // println!("Read time: {}", t.elapsed().as_micros());
                if *value == 64 {
                    self.layer.pc_stats.update_packets_read(1.0);
                    self.timestamp = Instant::now();
                }

                return *value;
            }
            _ => {
                // println!("No packet available");
                // *self.shutdown.write().unwrap() = true;
                // self.layer.control_flags.disconnect();
                self.layer.control_flags.initialize(false);
                self.reconnect();
            }
        }
        return 0;
    }

    /// After requesting a report this can be used to wait for a reply
    ///
    /// # Panics
    ///
    /// This will panic if a reply is not received from the Teensy
    /// within `timeout` ms.
    ///
    /// # Usage
    ///
    /// ```
    /// // set writer.output.data[0] to an int [1-3] (255 for initializer)
    /// writer.write();
    /// reader.wait_for_report_reply(255, 10);
    /// ```
    pub fn wait_for_report_reply(&mut self, packet_id: u8, timeout: u128) {
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

            // HID runs at 1 ms
            // self.layer.loop_delay(loopt);
        }

        // If packet never arrives
        self.layer.control_flags.shutdown();
        println!("HID Reader timed out waiting for reply from Teensy");
    }

    /// Main function to spin and connect the teensys
    /// input to ROS.
    ///
    /// # Usage
    /// ```
    /// use hidapi::HidApi;
    /// use dyse_rust::hid_comms::dyse_hid::HidReader;
    ///
    /// let mut hidapi = HidApi::new().expect("Failed to create API instance");
    /// let mut reader = HidReader::new(&mut hidapi, vid, pid);
    /// reader.spin();       // runs until watchdog times out
    /// ```
    pub fn spin(&mut self) {
        self.wait_for_report_reply(255, 100);

        while !self.layer.control_flags.is_shutdown() {
            let loopt = Instant::now();

            match self.read() {
                64 => {
                    // self.layer.report_parser(&self.input);
                    self.parser_tx.send(self.input.clone()).unwrap();
                }

                _ => {}
            }

            self.layer.delay(loopt);
            // if loopt.elapsed().as_micros() > 550 {
            //     println!(
            //         "HID Reader over cycled {}ms",
            //         1E-3 * (loopt.elapsed().as_micros() as f64)
            //     );
            // }
        }

        self.wait_for_report_reply(255, 100);
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
