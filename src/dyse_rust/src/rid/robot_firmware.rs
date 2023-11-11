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
    rid::{data_structures::*},
    socks::socks::*,
    utilities::{loaders::*},
};
use std::{time::Instant};
/// helpful constants to use
pub static P: u8 = 0x50;
pub static W: u8 = 0x57;
pub static M: u8 = 0x4D;
pub static D: u8 = 0x44;
pub static I: u8 = 0x4C;
pub static C: u8 = 0x53;
pub static DELIM: u8 = 0x3A;

/// HID laws
pub static MAX_HID_FLOAT_DATA: usize = 13;
pub static MAX_TASK_PARAMETERS: usize = 130;

/// first HID identifier
/// determines which report handler to use
/// # Usage
/// '''
///     packet.put(0, REPORT_ID);
/// '''
pub static INIT_REPORT_ID: u8 = 255; // Initialize
pub static TASK_CONTROL_ID: u8 = 1; // Request

/// second HID identifier for initializing
/// specifies the report hanlder mode
/// # Usage
/// '''
///     packet.put(0, INIT_REPORT_ID);
///     packet.put(1, REPORT_MODE);
/// '''
pub static INIT_NODE_MODE: u8 = 1;
pub static SETUP_CONFIG_MODE: u8 = 2;

/// second HID identifier for requests
/// specifies the report hanlder mode
/// # Usage
/// '''
///     packet.put(0, TASK_CONTROL_ID);
///     packet.put(1, REPORT_MODE);
/// '''
pub static OUTPUT_MODE: u8 = 2;

pub fn get_task_reset_packet(
    id: u8,
    rate: f64,
    driver: &Vec<u8>,
    input_ids: &Vec<u8>,
) -> HidPacket {
    let mut buffer = [0; HID_PACKET_SIZE];
    buffer[HID_MODE_INDEX] = INIT_REPORT_ID;
    buffer[HID_TOGL_INDEX] = INIT_NODE_MODE;
    buffer[HID_TASK_INDEX] = id;

    ((1E6 / rate) as u16)
    .to_le_bytes()
    .iter()
    .chain(driver.iter())
    .chain([input_ids.len() as u8].iter())
    .chain(input_ids.iter())
    .enumerate().for_each(|(i, &b)| buffer[HID_DATA_INDEX+i] = b);
    buffer
}

pub fn get_task_parameter_packets(id: u8, parameters: &Vec<f64>) -> Vec<HidPacket> {
    parameters
        .chunks(MAX_HID_FLOAT_DATA)
        .enumerate()
        .map(|(i, chunk)| {
            let mut buffer = [0; HID_PACKET_SIZE];
            buffer[HID_MODE_INDEX] = INIT_REPORT_ID;
            buffer[HID_TOGL_INDEX] = SETUP_CONFIG_MODE;
            buffer[HID_TASK_INDEX] = id;
            buffer[HID_DATA_INDEX] = i as u8;
            buffer[HID_DATA_INDEX+1] = chunk.len() as u8;
            chunk
                .into_iter()
                .map(|&x| (x as f32).to_le_bytes())
                .flatten()
                .enumerate()
                .for_each(|(i, b)| buffer[HID_DATA_INDEX+2+i] = b);

            buffer
        })
        .collect()
}

pub fn get_task_initializers(
    id: u8,
    rate: f64,
    driver: &Vec<u8>,
    parameters: &Vec<f64>,
    input_ids: &Vec<u8>,
) -> Vec<HidPacket> {
    [get_task_reset_packet(id, rate, driver, input_ids)]
        .into_iter()
        .chain(get_task_parameter_packets(id, parameters).into_iter())
        .collect()
}

pub struct EmbeddedTask {
    pub rate: f64,
    pub latch: u8,

    pub name: String,
    pub driver: String,

    pub parameters: Vec<f64>,

    pub input_names: Vec<String>,

    pub run_time: f64,
    pub pc_timestamp: f64,
    pub mcu_timestamp: f64,

    pub lifetime: Instant,
}

impl EmbeddedTask {
    pub fn named(
        name: String,
        driver: String,
        rate: f64,
        input_names: Vec<String>,
        params: Vec<f64>,
    ) -> EmbeddedTask {
        EmbeddedTask {
            rate: rate,
            latch: 0,

            name: name.clone(),
            driver: driver,

            parameters: params,

            input_names: input_names,

            run_time: 0.0,
            pc_timestamp: 0.0,
            mcu_timestamp: 0.0,

            lifetime: Instant::now(),
        }
    }

    pub fn set_params(&mut self, params: Vec<f64>) {
        if params.len() > MAX_TASK_PARAMETERS {
            panic!(
                "{:?} configuration exceeds maximum parameters ({})",
                self.name, MAX_TASK_PARAMETERS
            );
        }
        self.parameters = params;
    }

    pub fn driver(&self) -> Vec<u8> {
        self.driver.as_bytes().to_vec()
    }

    pub fn params(&self) -> Vec<u8> {
        self.parameters
            .iter()
            .map(|c| (*c as f32).to_be_bytes().to_vec())
            .flatten()
            .collect()
    }

    pub fn update(&mut self, latch: u8, run_time: f64, pc_time: f64, mcu_time: f64) {
        self.latch = latch;
        self.run_time = run_time;
        self.pc_timestamp = pc_time;
        self.mcu_timestamp = mcu_time;
    }

    pub fn print(&self) {
        println!("{:?}: {:?}", self.name, self.driver);
        println!(
            "\tRate: {}Hz\n\tRuntime: {}ms\n\tTimestamp(pc/mcu): {}s/{}s",
            self.rate, self.run_time, self.pc_timestamp, self.mcu_timestamp
        );
        println!("\tparameters:\n\t\t{:?}", self.parameters);
    }
}

pub struct TaskUpdate {
    pub task: String,
    pub mode: u8,
    pub data: Vec<u8>,
}

pub struct RobotFirmware {
    pub sock: Sock,
    pub configured: Vec<bool>,
    pub tasks: Vec<EmbeddedTask>,
}

impl RobotFirmware {
    pub fn from_byu(
        byu: BuffYamlUtil,
    ) -> RobotFirmware {
        let tasks: Vec<EmbeddedTask> = byu
            .data()
            .as_hash()
            .unwrap()
            .iter()
            .map(|(key, data)| {
                EmbeddedTask::named(
                    key.as_str().unwrap().to_string(),
                    byu.parse_str("driver", data).unwrap_or("NUL".to_string()),
                    byu.parse_float("rate", data).unwrap_or(-1.0),
                    byu.parse_strs("inputs", data).unwrap_or(vec![]),
                    byu.parse_floats("parameters", data).unwrap_or(vec![]),
                )
            })
            .collect();

        let target_names: Vec<String> = (0..tasks.len()).map(|i| tasks[i].name.clone() + "/ctrl").collect();

        RobotFirmware {
            sock: Sock::sinc("robot_fw", target_names.iter().map(|name| name.as_str()).collect()),
            configured: vec![false; tasks.len()],
            tasks: tasks,
        }
    }

    pub fn default() -> RobotFirmware {
        let byu = BuffYamlUtil::default("firmware_tasks");
        RobotFirmware::from_byu(byu)
    }

    pub fn new(
        robot_name: &str,
    ) -> RobotFirmware {
        let byu = BuffYamlUtil::robot(robot_name, "firmware_tasks");
        RobotFirmware::from_byu(byu)
    }

    pub fn from_self() -> RobotFirmware {
        let byu = BuffYamlUtil::from_self("firmware_tasks");
        RobotFirmware::from_byu(byu)
    }

    pub fn get_task_names(&self) -> Vec<&String> {
        self.tasks.iter().map(|task| &task.name).collect()
    }

    pub fn task_id(&self, name: &str) -> Option<usize> {
        self.tasks.iter().position(|task| name == task.name)
    }

    pub fn input_task_ids(&self, names: &Vec<String>) -> Vec<u8> {
        names
            .iter()
            .filter_map(|name| match self.task_id(name) {
                Some(i) => Some(i as u8),
                _ => None,
            })
            .collect()
    }

    pub fn task_init_packet(&self, idx: usize) -> Vec<HidPacket> {
        match self.configured[idx] {
            true => {
                vec![]
            }
            false => get_task_initializers(
                idx as u8,
                self.tasks[idx].rate,
                &self.tasks[idx].driver(),
                &self.tasks[idx].parameters,
                &self.input_task_ids(&self.tasks[idx].input_names),
            ),
        }
    }

    pub fn task_init_packets(&self) -> Vec<HidPacket> {
        (0..self.tasks.len())
            .map(|i| self.task_init_packet(i))
            .flatten()
            .collect()
    }

    pub fn task_param_packet(&self, idx: usize) -> Vec<HidPacket> {
        match self.configured[idx] {
            true => vec![],
            false => get_task_parameter_packets(idx as u8, &self.tasks[idx].parameters),
        }
    }

    pub fn task_param_packets(&self) -> Vec<HidPacket> {
        (0..self.tasks.len())
            .map(|i| self.task_param_packet(i))
            .flatten()
            .collect()
    }

    pub fn parse_sock(&mut self) {
        
    }

    pub fn parse_hid_feedback(&mut self, report: HidPacket, mcu_stats: &HidStats) {
        let rid = report[HID_MODE_INDEX];
        let mode = report[HID_TOGL_INDEX];
        let mcu_lifetime = f32::from_le_bytes([
            report[HID_TIME_INDEX],
            report[HID_TIME_INDEX+1],
            report[HID_TIME_INDEX+2],
            report[HID_TIME_INDEX+3]
        ]) as f64;

        let prev_mcu_lifetime = mcu_stats.lifetime();
        mcu_stats.set_lifetime(mcu_lifetime);

        let lifetime_diff = mcu_lifetime - prev_mcu_lifetime;
        if lifetime_diff >= 0.01 {
            println!("MCU Lifetime jump: {}", lifetime_diff);
        }

        if rid == INIT_REPORT_ID {
            if mode == INIT_REPORT_ID {
                mcu_stats.set_packets_sent(f32::from_le_bytes([
                    report[HID_TASK_INDEX],
                    report[HID_TASK_INDEX+1],
                    report[HID_TASK_INDEX+2],
                    report[HID_TASK_INDEX+3]
                ]) as f64);
                mcu_stats.set_packets_read(f32::from_le_bytes([
                    report[HID_TASK_INDEX+4],
                    report[HID_TASK_INDEX+5],
                    report[HID_TASK_INDEX+6],
                    report[HID_TASK_INDEX+7]
                ]) as f64);
            }
        } else if rid == TASK_CONTROL_ID && self.tasks.len() > mode as usize {
            // Zero length output packet means not configured
            // (only sends when task_node.is_configured() == false)
            let data_length = report[3] as usize;
            if data_length == 0 {
                self.configured[mode as usize] = false;
            } else if data_length < MAX_HID_FLOAT_DATA {

                let pc_time = 1E-6 * self.sock.lifetime.elapsed().as_micros() as f64;
                let runtime = f32::from_le_bytes([
                    report[HID_RUNT_INDEX],
                    report[HID_RUNT_INDEX+1],
                    report[HID_RUNT_INDEX+2],
                    report[HID_RUNT_INDEX+3]
                ]) as f64;

                let data: Vec<f64> = report[4..4+(4*data_length)].to_vec().chunks(4).map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()) as f64).collect();
                self.sock.tx_any_payload(&self.tasks[mode as usize].name, data, (1E6 / self.tasks[mode as usize].rate) as u64);
                self.tasks[mode as usize].update(report[2], runtime, pc_time, mcu_lifetime);
                self.configured[mode as usize] = true;

            } else {
                println!("[Robot-Firmware]: unknown report {report:?}");
            }

            mcu_stats.update_packets_sent(1.0); // only works if we don't miss packets
            mcu_stats.update_packets_read(1.0);
        }
    }

    pub fn print(&self) {
        println!("[Robot-Firmware]: {:?}", self.configured);
        self.tasks.iter().enumerate().for_each(|(i, task)| {
            println!("===== Task [{}] =====", i);
            task.print();
        });
    }
}
