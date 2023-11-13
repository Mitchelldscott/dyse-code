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
use dyse_rust::{
    rid::robot_firmware::TaskCommunication,
    socks::{message::UDP_PACKET_SIZE, socks::Sock},
};
use pyo3::{prelude::*, types::PyDict};
use std::time::Instant;

#[pyfunction]
fn send(name: &str, data: Vec<u8>) -> PyResult<()> {
    let mut sock = Sock::source(&name);
    sock.tx_payload(data);
    Ok(())
}

#[pyclass]
struct PySock {
    sock: Sock,
}

#[pymethods]
impl PySock {
    #[new]
    fn new(name: &str, targets: Vec<&str>) -> Self {
        PySock {
            sock: Sock::sinc(name, targets),
        }
    }

    pub fn send(&mut self, data: Vec<f64>) -> PyResult<()> {
        self.sock.tx_payload(data);
        Ok(())
    }

    pub fn recv_f64(&mut self) -> PyResult<Vec<Vec<f64>>> {
        let mut buffer = [0u8; UDP_PACKET_SIZE];
        match self.sock.try_rx(&mut buffer) {
            Some(_) => Ok(self
                .sock
                .recv_available()
                .iter()
                .map(|payload| {
                    bincode::deserialize::<Vec<f64>>(payload)
                        .expect("Failed to serialize payload (pysock)")
                })
                .collect()),
            _ => Ok(vec![]),
        }
    }
    // pub struct TaskCommunication {
    //     pub name: String,
    //     pub latch: u8,
    //     pub rate: f64,
    //     pub pc_time: f64,
    //     pub mcu_time: f64,
    //     pub run_time: f64,
    //     pub data: Vec<f6
    // }
    pub fn recv_fw(&mut self) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let t = Instant::now();

            let mut rx = false;
            let fw_dict = PyDict::new(py);
            let mut buffer = [0u8; UDP_PACKET_SIZE];

            while !rx && t.elapsed().as_secs() < 1 && !*self.sock.shutdown.read().unwrap() {
                match self.sock.try_rx(&mut buffer) {
                    Some(_) => {
                        self.sock.recv_available().iter().for_each(|payload| {
                            let task_dict = PyDict::new(py);
                            let packet = bincode::deserialize::<TaskCommunication>(payload)
                                .expect("Failed to serialize payload (pysock)");

                            task_dict.set_item("latch", packet.latch).unwrap();
                            task_dict.set_item("rate", packet.rate).unwrap();
                            task_dict.set_item("pc_time", packet.pc_time).unwrap();
                            task_dict.set_item("mcu_time", packet.mcu_time).unwrap();
                            task_dict.set_item("run_time", packet.run_time).unwrap();
                            task_dict.set_item("data", packet.data).unwrap();

                            fw_dict.set_item(packet.name, task_dict).unwrap();
                            rx = true;
                        });
                    }
                    _ => {}
                }
            }

            Ok(fw_dict.into())
        })
    }
}

#[pymodule]
fn socks(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(send, m)?)?;
    m.add_class::<PySock>()?;
    Ok(())
}
