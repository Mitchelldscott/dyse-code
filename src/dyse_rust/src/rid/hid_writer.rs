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

use crate::{rid::hid_layer::*, utilities::data_structures::*};

use crossbeam_channel::Receiver;
use hidapi::HidDevice;
use std::time::Instant;

pub struct HidWriter {
    writer_rx: Receiver<HidPacket>,
    teensy: HidDevice,
    layer: HidLayer,
    timestamp: Instant,
}

impl HidWriter {
    pub fn new(layer: HidLayer, writer_rx: Receiver<HidPacket>) -> HidWriter {
        HidWriter {
            writer_rx: writer_rx,
            teensy: layer.wait_for_device(),
            layer: layer,
            timestamp: Instant::now(),
        }
    }

    pub fn print(&self) {
        println!(
            "Writer Dump\n\ttimer: {} us\n\tpackets: {}",
            self.timestamp.elapsed().as_micros(),
            self.layer.pc_stats.packets_sent(),
        );
        self.output.print();
    }

    pub fn silent_channel_default(&mut self) -> HidPacket {
        let mut buffer = [0; HID_PACKET_SIZE];
        buffer[0] = 255;
        buffer[1] = 255;
        (self.layer.pc_stats.packets_sent() as f32).to_be_bytes().iter().enumerate().for_each(|(i, b)| buffer[i+2] = *b);
        buffer
    }

    pub fn reconnect(&mut self) {
        // check reconnect after 1000 cycles
        if !self.layer.control_flags.is_shutdown()
            && self.timestamp.elapsed().as_millis() as f64 > self.layer.sample_time
        {
            if self.layer.control_flags.is_connected() {
                println!(
                    "[HID-Writer]: disconnecting, hasn't written for {}s",
                    (self.timestamp.elapsed().as_millis() as f64) * 1E-3
                );

                self.layer.control_flags.disconnect();
            }

            self.teensy = self.layer.wait_for_device();
        }
    }

    /// Write the bytes from the buffer to the teensy.
    /// Reconnect if the write fails.
    /// # Usage
    /// ```
    /// let mut buffer = [0; HID_PACKET_SIZE];
    /// let writer = HidWriter::new(layer, writer_rx);
    /// writer.write(buffer); // writes some_data to the teensy
    /// ```
    pub fn write(&mut self, buffer: HidPacket) {
        (1E-3 * (lifetime.elapsed().as_micros() as f32)).to_be_bytes().iter().enumerate().for_each(|(i, b)| buffer[i + TIMESTAMP_OFFSET] = *b);

        match self.teensy.write(buffer) {
            Ok(value) => {
                self.timestamp = Instant::now();
                if value == HID_PACKET_SIZE {
                    self.layer.pc_stats.update_packets_sent(1.0);
                }
            }
            _ => {
                self.reconnect();
            }
        }
    }

    /// Continually sends data from 'writer_rx' to the teensy.
    ///
    ///
    /// # Example
    /// See [`HidLayer::pipeline()`] source
    pub fn pipeline(&mut self) {
        let lifetime = Instant::now();
        println!("[HID-writer]: Live");

        while !self.layer.control_flags.is_shutdown() {
            let t = Instant::now();

            let mut buffer = self.writer_rx.try_recv(&mut buffer).unwrap_or(self.silent_channel_default());

            self.write(buffer);

            self.layer.delay(t);
        }

        self.send_report(13, &vec![255; 63]);

        println!("[HID-writer]: Shutdown");
    }
}
