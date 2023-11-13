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

use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};

pub const HID_PACKET_SIZE: usize = 64;

pub const HID_MODE_INDEX: usize = 0; // 255 = init data, 1 = overwrite data, 13 = kill
pub const HID_TOGL_INDEX: usize = 1; // init data: (1 = init task, 2 = config task) overwrite data: (latch)
pub const HID_TASK_INDEX: usize = 2; // only applies to init/overwrite data
pub const HID_DATA_INDEX: usize = 3; // data start
pub const HID_RUNT_INDEX: usize = 48;
pub const HID_RUCT_INDEX: usize = 52;
pub const HID_PCTS_INDEX: usize = 56;
pub const HID_UCTS_INDEX: usize = 60;

pub type HidPacket = [u8; HID_PACKET_SIZE];

#[derive(Clone)]
pub struct NetFlowStats {
    ntx: Arc<RwLock<f64>>,
    nrx: Arc<RwLock<f64>>,
    t: Arc<RwLock<f64>>,
}

impl NetFlowStats {
    pub fn new() -> NetFlowStats {
        NetFlowStats {
            ntx: Arc::new(RwLock::new(0.0)),
            nrx: Arc::new(RwLock::new(0.0)),
            t: Arc::new(RwLock::new(0.0)),
        }
    }

    pub fn n_tx(&self) -> f64 {
        *self.ntx.read().unwrap()
    }

    pub fn update_tx(&self, n: f64) {
        *self.ntx.write().unwrap() += n;
    }

    pub fn set_tx(&self, n: f64) {
        *self.ntx.write().unwrap() = n;
    }

    pub fn n_rx(&self) -> f64 {
        *self.nrx.read().unwrap()
    }

    pub fn update_rx(&self, n: f64) {
        *self.nrx.write().unwrap() += n;
    }

    pub fn set_rx(&self, n: f64) {
        *self.nrx.write().unwrap() = n;
    }

    pub fn time(&self) -> f64 {
        *self.t.read().unwrap()
    }

    pub fn from_utc(&self, datetime: DateTime<Utc>) -> f64 {
        let t = 1E-6 * (Utc::now().timestamp_micros() - datetime.timestamp_micros()) as f64;
        *self.t.write().unwrap() = t;
        t
    }

    pub fn from_utcs(&self, datetime1: DateTime<Utc>, datetime2: DateTime<Utc>) -> f64 {
        let t = 1E-6 * (datetime1.timestamp_micros() - datetime2.timestamp_micros()) as f64;
        *self.t.write().unwrap() = t;
        t
    }

    pub fn from_bytes(&self, bytes: &[u8]) -> f64 {
        let t = f32::from_le_bytes(bytes.try_into().unwrap()) as f64;
        *self.t.write().unwrap() = t;
        t
    }

    pub fn set_time(&self, t: f64) {
        *self.t.write().unwrap() = t;
    }

    pub fn print(&self) {
        println!("\tPackets Tx/Rx: {}/{}", self.n_tx(), self.n_rx());
    }
}

#[derive(Clone)]
pub struct HidControlFlags {
    // Logic flags to cause events in other threads
    shutdown: Arc<RwLock<bool>>,
    connected: Arc<RwLock<bool>>,
    initialized: Arc<RwLock<bool>>,
}

impl HidControlFlags {
    pub fn new() -> HidControlFlags {
        HidControlFlags {
            shutdown: Arc::new(RwLock::new(false)),
            connected: Arc::new(RwLock::new(false)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        *self.shutdown.read().unwrap()
    }

    pub fn shutdown(&self) {
        *self.shutdown.write().unwrap() = true;
    }

    pub fn startup(&self) {
        *self.shutdown.write().unwrap() = false;
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.read().unwrap()
    }

    pub fn connect(&self) {
        *self.connected.write().unwrap() = true;
    }

    pub fn disconnect(&self) {
        *self.connected.write().unwrap() = false;
    }

    pub fn is_initialized(&self) -> bool {
        *self.initialized.read().unwrap()
    }

    pub fn initialize(&self, status: bool) {
        *self.initialized.write().unwrap() = status;
    }

    pub fn print(&self) {
        println!(
            "\tShutdown: {}\n\tConnected: {}\n\tInitialized: {}",
            self.is_shutdown(),
            self.is_connected(),
            self.is_initialized()
        );
    }
}
