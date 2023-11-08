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
    utilities::{data_structures::*, loaders::*},
};
use std::{thread::JoinHandle, time::Instant};
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
) -> ByteBuffer {
    let mut buffer = ByteBuffer::hid();
    buffer.puts(0, &vec![INIT_REPORT_ID, INIT_NODE_MODE, id]);
    buffer.puts(3, &((1E6 / rate) as u16).to_le_bytes().to_vec());
    buffer.puts(5, driver);
    buffer.put(10, input_ids.len() as u8);
    buffer.puts(11, input_ids);
    buffer
}

pub fn get_task_parameter_packets(id: u8, parameters: &Vec<f64>) -> Vec<ByteBuffer> {
    parameters
        .chunks(MAX_HID_FLOAT_DATA)
        .enumerate()
        .map(|(i, chunk)| {
            let mut buffer = ByteBuffer::hid();
            buffer.puts(
                0,
                &vec![
                    INIT_REPORT_ID,
                    SETUP_CONFIG_MODE,
                    id,
                    i as u8,
                    chunk.len() as u8,
                ],
            );

            buffer.puts(
                5,
                &chunk
                    .into_iter()
                    .map(|x| (*x as f32).to_le_bytes())
                    .flatten()
                    .collect(),
            );

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
) -> Vec<ByteBuffer> {
    [get_task_reset_packet(id, rate, driver, input_ids)]
        .into_iter()
        .chain(get_task_parameter_packets(id, parameters).into_iter())
        .collect()
}

pub struct EmbeddedTask {
    pub rate: f64,
    pub latch: u16,

    pub name: String,
    pub driver: String,

    pub parameters: Vec<f64>,

    pub input_names: Vec<String>,

    pub run_time: f64,
    pub timestamp: f64,

    pub sock: Sock,
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
            timestamp: 0.0,

            sock: Sock::source(&name),
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

    pub fn broadcast(&mut self, latch: u16, run_time: f64, timestamp: f64, data: Vec<f64>) {
        self.latch = latch;
        self.run_time = run_time;
        self.timestamp = timestamp;
        self.sock.send(data);
    }

    pub fn print(&self) {
        println!("{:?}: {:?}", self.name, self.driver);
        println!(
            "\tRate: {}Hz\n\tRuntime: {}ms\n\tTimestamp: {}s",
            self.rate, self.run_time, self.timestamp
        );
        println!("\tparameters:\n\t\t{:?}", self.parameters);
    }
}

pub struct TaskParams {
    task: usize,
    params: Vec<f64>,
}

pub struct RobotFirmware {
    pub configured: Vec<bool>,
    pub tasks: Vec<EmbeddedTask>,
    pub params: crossbeam_channel::Receiver<TaskParams>,
    pub latch_handles: Vec<JoinHandle<()>>,
    pub config_handles: Vec<JoinHandle<()>>,
}

impl RobotFirmware {
    pub fn from_byu(
        byu: BuffYamlUtil,
        writer_tx: crossbeam_channel::Sender<HidPacket>,
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

        let mut rfw = RobotFirmware {
            configured: vec![false; tasks.len()],
            tasks: tasks,
            latch_handles: vec![],
            config_handles: vec![],
        };

        let task_config_topics = (0..tasks.len()).map(|i| format!("{}/params", tasks[i].name)).collect();

        let mut sock = Sock::unsynced("robot_fw", task_config_topics, "", 0, |data: Vec<UdpPayload>, _ctx: &mut UdpPayload, t: f64| {
            (0, data[0])
        });

    }

    pub fn default(writer_tx: crossbeam_channel::Sender<ByteBuffer>) -> RobotFirmware {
        let byu = BuffYamlUtil::default("firmware_tasks");
        RobotFirmware::from_byu(byu, writer_tx)
    }

    pub fn new(
        robot_name: &str,
        writer_tx: crossbeam_channel::Sender<ByteBuffer>,
    ) -> RobotFirmware {
        let byu = BuffYamlUtil::robot(robot_name, "firmware_tasks");
        RobotFirmware::from_byu(byu, writer_tx)
    }

    pub fn from_self(writer_tx: crossbeam_channel::Sender<ByteBuffer>) -> RobotFirmware {
        let byu = BuffYamlUtil::from_self("firmware_tasks");
        RobotFirmware::from_byu(byu, writer_tx)
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

    pub fn task_init_packet(&self, idx: usize) -> Vec<ByteBuffer> {
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

    pub fn task_init_packets(&self) -> Vec<ByteBuffer> {
        (0..self.tasks.len())
            .map(|i| self.task_init_packet(i))
            .flatten()
            .collect()
    }

    pub fn task_param_packet(&self, idx: usize) -> Vec<ByteBuffer> {
        match self.configured[idx] {
            true => vec![],
            false => get_task_parameter_packets(idx as u8, &self.tasks[idx].parameters),
        }
    }

    pub fn task_param_packets(&self) -> Vec<ByteBuffer> {
        (0..self.tasks.len())
            .map(|i| self.task_param_packet(i))
            .flatten()
            .collect()
    }

    pub fn parse_hid_feedback(&mut self, report: ByteBuffer, mcu_stats: &HidStats) {
        let rid = report.get(0);
        let mode = report.get(1);
        let mcu_lifetime = report.get_float(60);
        let prev_mcu_lifetime = mcu_stats.lifetime();
        mcu_stats.set_lifetime(mcu_lifetime);

        let lifetime_diff = mcu_lifetime - prev_mcu_lifetime;
        if lifetime_diff >= 0.01 {
            println!("MCU Lifetime jump: {}", lifetime_diff);
        }

        if rid == INIT_REPORT_ID {
            if mode == INIT_REPORT_ID {
                // let prev_mcu_write_count = mcu_stats.packets_sent();
                // let packet_diff = report.get_float(2) - prev_mcu_write_count;
                // if packet_diff > 1.0 {
                //     println!("MCU Packet write difference: {}", packet_diff);
                // }

                mcu_stats.set_packets_sent(report.get_float(2));
                mcu_stats.set_packets_read(report.get_float(6));
            }
        } else if rid == TASK_CONTROL_ID && self.tasks.len() > mode as usize {
            // Zero length output packet means not configured
            // (only sends when task_node.is_configured() == false)
            if report.get(3) == 0 {
                self.configured[mode as usize] = false;
            } else if report.get(3) < MAX_HID_FLOAT_DATA as u8 {
                self.configured[mode as usize] = true;

                self.tasks[mode as usize].broadcast(
                    report.get(2) as u16,
                    report.get_float(56),
                    mcu_lifetime,
                    report.gets(4, report.get(3) as usize),
                );
            } else {
                report.print();
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
