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

use crate::{comms::hid_layer::*, utilities::data_structures::*};

use crossbeam_channel::Receiver;
use hidapi::HidDevice;
use std::time::Instant;

pub struct HidWriter {
    writer_rx: Receiver<ByteBuffer>,
    output: ByteBuffer,
    teensy: HidDevice,
    layer: HidLayer,
    timestamp: Instant,
}

impl HidWriter {
    pub fn new(layer: HidLayer, writer_rx: Receiver<ByteBuffer>) -> HidWriter {
        HidWriter {
            writer_rx: writer_rx,
            output: ByteBuffer::hid(),
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

    pub fn buffer(&self) -> &ByteBuffer {
        &self.output
    }

    pub fn silent_channel_default(&mut self) -> ByteBuffer {
        let mut buffer = ByteBuffer::hid();
        buffer.puts(0, vec![255, 255]);
        buffer.put_float(2, self.layer.pc_stats.packets_sent());
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

    /// Write the bytes from the output buffer to the teensy, then clear the buffer.
    /// Shutdown if the write fails.
    /// # Usage
    /// ```
    /// writer.output.puts(some_index, some_data);
    /// writer.write(); // writes some_data to the teensy
    /// ```
    pub fn write(&mut self) {
        match self.teensy.write(&self.output.data) {
            Ok(value) => {
                // self.output.reset();
                self.timestamp = Instant::now();
                if value == 64 {
                    self.layer.pc_stats.update_packets_sent(1.0);
                }
            }
            _ => {
                self.reconnect();
            }
        }
    }

    /// Creates a report from `id` and `data` and sends it to the teensy. Only use in testing.
    /// # Usage
    /// ```
    ///     writer.teensy = hidapi.open(vid, pid);
    ///     writer.send_report(report_id, data);
    /// ```
    pub fn send_report(&mut self, id: u8, data: Vec<u8>) {
        self.output.puts(0, vec![id]);
        self.output.puts(1, data);
        self.write();
    }

    /// Continually sends data from [HidROS] (or whatever owns the other end of `writer_rx`) to the teensy.
    ///
    /// # Arguments
    /// * `shutdown` - The function stops when this is true.
    /// Used so that HidLayer threads, all running pipeline() at the same time, can be shutdown at the same time (by passing them the same variable)
    /// * `writer_rx` - Receives the data from [HidROS].
    ///
    /// # Example
    /// See [`HidLayer::pipeline()`] source
    pub fn pipeline(&mut self) {
        let lifetime = Instant::now();
        println!("[HID-writer]: Live");

        while !self.layer.control_flags.is_shutdown() {
            let t = Instant::now();

            self.output = self
                .writer_rx
                .try_recv()
                .unwrap_or(self.silent_channel_default());

            self.output
                .put_float(60, 1E-3 * (lifetime.elapsed().as_micros() as f64));

            self.write();

            self.layer.delay(t);
            // t = Instant::now(); // try double delay
            // self.layer.delay(t);
            // if t.elapsed().as_micros() > 550 {
            //     println!(
            //         "HID Writer over cycled {}ms",
            //         1E-3 * (t.elapsed().as_micros() as f64)
            //     );
            // }
        }

        let kill_switch = vec![255; 63];
        self.send_report(13, kill_switch);

        println!("[HID-writer]: Shutdown");
    }
}
