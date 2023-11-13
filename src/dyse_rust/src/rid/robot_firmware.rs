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
    rid::data_structures::*,
    socks::{message::UDP_PACKET_SIZE, socks::*},
    utilities::loaders::*,
};
use serde::{Deserialize, Serialize};

/// helpful constants to use
pub static P: u8 = 0x50;
pub static W: u8 = 0x57;
pub static M: u8 = 0x4D;
pub static D: u8 = 0x44;
pub static I: u8 = 0x4C;
pub static C: u8 = 0x53;
pub static DELIM: u8 = 0x3A;

/// HID laws
pub static MAX_HID_FLOAT_DATA: usize = 10;
pub static MAX_TASK_PARAMETERS: usize = 100;

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
        .enumerate()
        .for_each(|(i, &b)| buffer[HID_DATA_INDEX + i] = b);
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
            buffer[HID_DATA_INDEX + 1] = chunk.len() as u8;
            chunk
                .into_iter()
                .map(|&x| (x as f32).to_le_bytes())
                .flatten()
                .enumerate()
                .for_each(|(i, b)| buffer[HID_DATA_INDEX + 2 + i] = b);

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

pub fn get_latch_packet(i: u8, latch: u8, data: &Vec<f64>) -> HidPacket {
    let mut buffer = [0; HID_PACKET_SIZE];
    buffer[HID_MODE_INDEX] = TASK_CONTROL_ID;
    buffer[HID_TOGL_INDEX] = latch;
    buffer[HID_TASK_INDEX] = i;
    buffer[HID_DATA_INDEX] = data.len() as u8;
    data.iter().enumerate().for_each(|(i, &x)| {
        (x as f32)
            .to_le_bytes()
            .iter()
            .enumerate()
            .for_each(|(j, &b)| buffer[(4 * i) + j] = b)
    });
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskMarshallType {
    Input,
    Output,
    Parameter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskMarshall {
    pub name: String,
    pub data: Vec<f64>,
    pub mode: TaskMarshallType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskCommunication {
    pub name: String,
    pub latch: u8,
    pub rate: f64,
    pub pc_time: f64,
    pub mcu_time: f64,
    pub run_time: f64,
    pub data: Vec<f64>,
}

pub struct EmbeddedTask {
    pub name: String,
    pub driver: String,

    pub input_names: Vec<String>,

    pub output: Vec<f64>,
    pub parameters: Vec<f64>,

    pub latch: u8,
    pub rate: f64,
    pub pc_time: f64,
    pub mcu_time: f64,
    pub run_time: f64,
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
            name: name.clone(),
            driver: driver,

            output: vec![],
            parameters: params,
            input_names: input_names,

            latch: 0,
            rate: rate,
            pc_time: 0.0,
            mcu_time: 0.0,
            run_time: 0.0,
        }
    }

    pub fn set_params(&mut self, params: Vec<f64>) {
        if params.len() > MAX_TASK_PARAMETERS {
            panic!(
                "{:?} configuration exceeds maximum parameters ({MAX_TASK_PARAMETERS})",
                self.name
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
            .map(|c| (*c as f32).to_le_bytes().to_vec())
            .flatten()
            .collect()
    }

    pub fn update(&mut self, packet: TaskCommunication) {
        // let mcu_diff = packet.mcu_time - self.mcu_time;

        // let new_diff = packet.pc_time - packet.mcu_time;
        // let prev_diff = self.pc_time - self.mcu_time;

        // let diff_ratio = new_diff.abs() / prev_diff.abs(); // ratio of time differences, want = 1
        // let n_missed: u8 = ((self.rate * mcu_diff) + 0.5) as u8; // average time * rate = estimated samples

        // if n_missed > 1 {
        //     println!("[{}] missed {:.1} packets {:.4}s {:.4}s", self.name, n_missed, packet.pc_time - self.pc_time, mcu_diff);
        // }
        // // if diff_ratio < 1.0 - 1E-6 || diff_ratio > 1.0 + 1E-6 {
        // if (diff_ratio - 1.0).abs() > 1E-6 {
        //     println!("[{}] time drift delta: {:.6}s, drift: {:.6}s", self.name, new_diff - prev_diff, new_diff)
        // }
        // if mcu_diff < 0.95 / self.rate {
        //     println!("[{}] is running to fast {:.6}s", self.name, mcu_diff)
        // }

        self.latch = packet.latch;
        self.output = packet.data;
        self.pc_time = packet.pc_time;
        self.mcu_time = packet.mcu_time;
        self.run_time = packet.run_time;
    }

    pub fn print(&self) {
        println!("{:?}: {:?}", self.name, self.driver);
        println!(
            "\tRate: {}Hz\n\tRuntime: {}ms\n\tTimestamp(pc/mcu): {}s/{}s",
            self.rate, self.run_time, self.pc_time, self.mcu_time
        );
        println!("\tparameters:\n\t\t{:?}", self.parameters);
    }
}

pub struct RobotFirmware {
    pub sock: Sock,
    pub configured: Vec<bool>,
    pub tasks: Vec<EmbeddedTask>,
}

impl RobotFirmware {
    pub fn from_byu(byu: BuffYamlUtil) -> RobotFirmware {
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

        let target_names: Vec<String> = (0..tasks.len())
            .map(|i| tasks[i].name.clone() + "/ctrl")
            .collect();

        RobotFirmware {
            sock: Sock::sinc(
                "robot_fw",
                target_names.iter().map(|name| name.as_str()).collect(),
            ),
            configured: vec![false; tasks.len()],
            tasks: tasks,
        }
    }

    pub fn default() -> RobotFirmware {
        let byu = BuffYamlUtil::default("firmware_tasks");
        RobotFirmware::from_byu(byu)
    }

    pub fn new(robot_name: &str) -> RobotFirmware {
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

    pub fn unconfigured_tasks(&self) -> Vec<usize> {
        (0..self.configured.len())
            .filter(|&i| self.configured[i])
            .collect()
    }

    pub fn all_init_packets(&self) -> Vec<HidPacket> {
        (0..self.tasks.len())
            .map(|i| {
                get_task_initializers(
                    i as u8,
                    self.tasks[i].rate,
                    &self.tasks[i].driver(),
                    &self.tasks[i].parameters,
                    &self.input_task_ids(&self.tasks[i].input_names),
                )
            })
            .flatten()
            .collect()
    }

    pub fn unconfigured_parameters(&self) -> Vec<HidPacket> {
        self.unconfigured_tasks()
            .into_iter()
            .map(|i| get_task_parameter_packets(i as u8, &self.tasks[i].parameters))
            .flatten()
            .collect()
    }

    pub fn parse_sock(&mut self) -> Option<HidPacket> {
        let mut buffer = [0u8; UDP_PACKET_SIZE];
        match self.sock.try_rx(&mut buffer) {
            Some(i) => {
                let packet: TaskMarshall =
                    bincode::deserialize(&self.sock.messages[i].to_payload()).unwrap();
                match self.sock.is_target(&packet.name) {
                    Some(i) => match packet.mode {
                        TaskMarshallType::Input => Some(input_latch(i as u8, &packet.data)),
                        TaskMarshallType::Output => Some(output_latch(i as u8, &packet.data)),
                        TaskMarshallType::Parameter => {
                            self.configured[i] = false;
                            self.tasks[i].set_params(packet.data);
                            None
                        }
                    },
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn parse_hid(
        &mut self,
        pc_time: f64,
        mcu_time: f64,
        run_time: f64,
        task_idx: usize,
        buffer: HidPacket,
    ) {
        if self.tasks.len() > task_idx {
            let data_length = buffer[HID_DATA_INDEX] as usize;

            if data_length == 0 {
                // tasks send 0 length data when unconfigured

                self.configured[task_idx] = false;
            } else if data_length < MAX_HID_FLOAT_DATA {
                // garbage shield

                let comm_packet = TaskCommunication {
                    name: format!("{}", self.tasks[task_idx].name),
                    latch: buffer[HID_TOGL_INDEX],
                    rate: 1.0 / (mcu_time - self.tasks[task_idx].mcu_time),
                    pc_time: pc_time,
                    mcu_time: mcu_time,
                    run_time: run_time,
                    data: buffer[4..4 + (4 * data_length)]
                        .to_vec()
                        .chunks(4)
                        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()) as f64)
                        .collect(),
                };

                // println!("{} {}", self.tasks[task_idx].name, (comm_packet.pc_time - self.tasks[task_idx].pc_time + comm_packet.mcu_time - self.tasks[task_idx].mcu_time) / 2.0);

                self.sock.tx_any_payload(
                    &comm_packet.name,
                    &comm_packet,
                    (1E6 / self.tasks[task_idx].rate) as u64,
                );

                self.tasks[task_idx].update(comm_packet);

                self.configured[task_idx] = true;
            } else {
                println!("[Robot-Firmware]: garbage report {buffer:?}");
            }
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
