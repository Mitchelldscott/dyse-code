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

use std::{
    fmt,
    time::Instant,
};
use crate::socks::{
    message::{
        UdpPayload,
    },
};

pub const TASK_SUCCESS:         usize = 0;
pub const TASK_WARN:            usize = 1;
pub const TASK_ERROR:           usize = 2;
pub const TASK_IO_ERROR:        usize = 3;
pub const TASK_UNIMPLEMENTED:   usize = 4;
pub const TASK_LABELS:      [&str; 5] = ["", "WARN", "ERROR", "IO_ERROR", "UNIMPLEMENTED"];

pub trait GenExecutable<T, U, V>: Fn(T, &mut U, f64) -> (usize, V) {}
impl<F, T, U, V> GenExecutable<T, U, V> for F where F: Fn(T, &mut U, f64) -> (usize, V) {}

pub trait TaskExecutable: GenExecutable<Vec<UdpPayload>, UdpPayload, UdpPayload> {}
impl<F> TaskExecutable for F where F: GenExecutable<Vec<UdpPayload>, UdpPayload, UdpPayload> {}
// Default types and implementations
pub type TaskFn = fn(Vec<UdpPayload>, &mut UdpPayload, f64) -> (usize, UdpPayload);
pub fn empty_exe(_:Vec<UdpPayload>, _:&mut UdpPayload, _:f64) -> (usize, UdpPayload) {
    (0, vec![])
}


#[macro_export]
macro_rules! build_fn {
    (|$context:ident: $U:ty, $($target:ident: $T:ty),+| $body:expr) => (
        build_fn!(|$context: $U, _t, $($target: $T),+| $body)
    );
    // makes the timestamp function is called with accessible
    (|$context:ident: $U:ty, $time:ident, $target:ident: $T:ty| $body:expr) => (
        |task_input: Vec<Vec<u8>>, task_context: &mut Vec<u8>, $time: f64| -> (usize, Vec<u8>) {
            let return_code = 0;
            #[allow(unused_mut)]
            let mut $context: $U = bincode::deserialize(&task_context).expect("Failed to deserialize context");
            let $target: Vec<$T> = task_input.iter().map(|task_in| bincode::deserialize::<$T>(&task_in).expect("Failed to deserialize input")).collect();

            let output = $body;
            *task_context = bincode::serialize(&$context).expect("Failed to serialize context");
            (return_code, bincode::serialize(&output).expect("Failed to serialize output"))
        }
    );
    (|$context:ident: $U:ty, $time:ident, $($target:ident: $T:ty),+| $body:expr) => (
        |task_input: Vec<Vec<u8>>, task_context: &mut Vec<u8>, $time: f64| -> (usize, Vec<u8>) {
            let mut argc = 0;
            let return_code = 0;
            #[allow(unused_mut)]
            let mut $context: $U = bincode::deserialize(&task_context).unwrap();
            $(
                let $target: $T = bincode::deserialize(&task_input[argc]).unwrap();
                argc += 1;
            )+

            let output = $body;
            *task_context = bincode::serialize(&$context).unwrap();
            (return_code, bincode::serialize(&output).unwrap())
        }
    );
}

#[derive(Debug, Clone)]
pub struct TaskError {
    pub data: String,
}

impl TaskError {
    pub fn default(name: &str, code: usize, timestamp: f64) -> TaskError {
        let label = TASK_LABELS[code];

        TaskError {
            data: format!("[TASK {label}]({timestamp}s):{name}"),
        }
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.data)
    }
}

pub struct Task {
    pub timestamp: Instant,
    pub lifetime: Instant,

    pub name: String,
    pub targets: Vec<usize>,

    context: UdpPayload,
    task: TaskFn,
}

impl Task {
    pub fn new(name: &str, targets: Vec<usize>, context: UdpPayload, task: TaskFn) -> Task {

        Task {
            timestamp: Instant::now(),
            lifetime: Instant::now(),

            name: name.to_string(),
            targets: targets,

            context: context,
            task: task,
        }
    }

    pub fn execute(&mut self, data: Vec<UdpPayload>) -> Result<UdpPayload, TaskError> {

        self.timestamp = Instant::now();

        let t = self.lifetime.elapsed().as_micros() as f64 * 1E-6;
        let (code, output) = (self.task)(data, &mut self.context, t);

        match code {
            TASK_SUCCESS => Ok(output),
            _ => Err(TaskError::default(&self.name, code as usize, t)),
        }
    }
}
