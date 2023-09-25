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

use ::std::time::Instant;

/// buffer for byte packet storage
///  stores the data, timestamp of the last access
///  and a flag for updates.
#[derive(Clone)]
pub struct ByteBuffer {
    pub data: Vec<u8>,      // data packet
    pub timestamp: Instant, // for time sensitive things
    pub update_flag: bool,  // an update flag for updates
}

impl ByteBuffer {
    pub fn new(n: usize) -> ByteBuffer {
        /*
            Create a new buffer.
        */
        ByteBuffer {
            data: vec![0; n],
            update_flag: false,
            timestamp: Instant::now(),
        }
    }

    pub fn from_vec(v: Vec<u8>) -> ByteBuffer {
        ByteBuffer {
            data: v,
            update_flag: false,
            timestamp: Instant::now(),
        }
    }

    pub fn hid() -> ByteBuffer {
        /*
            Create a new 64 byte buffer.
        */
        ByteBuffer::new(64)
    }

    pub fn tcp() -> ByteBuffer {
        /*
            Create a new 1024 byte buffer.
        */
        ByteBuffer::new(1024)
    }

    pub fn buffer(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn validate_index(&self, idx: usize) {
        if idx >= self.data.len() {
            panic!("Invalid index for operation {} < {}", idx, self.data.len());
        }
    }

    pub fn get(&self, idx: usize) -> u8 {
        /*
            Read a value in the buffer.
        */
        self.validate_index(idx);
        self.data[idx]
    }

    pub fn put(&mut self, idx: usize, value: u8) {
        /*
            Write a value in the buffer.
        */
        self.validate_index(idx);
        self.data[idx] = value;
    }

    pub fn get_i32(&self, idx: usize) -> i32 {
        /*
            Read an i32 from the buffer.
        */
        self.validate_index(idx);
        self.validate_index(idx + 3);
        i32::from_le_bytes([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
    }

    pub fn get_u32(&self, idx: usize) -> u32 {
        /*
            Read an i32 from the buffer.
        */
        self.validate_index(idx);
        self.validate_index(idx + 3);
        u32::from_le_bytes([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
    }

    pub fn put_i32(&mut self, idx: usize, value: i32) {
        /*
            Write an i32 to the buffer.
        */
        self.puts(idx, value.to_le_bytes().to_vec());
    }

    pub fn get_float(&self, idx: usize) -> f64 {
        /*
            Read an f32 from the buffer.
            return as f64 just because.
        */
        self.validate_index(idx);
        self.validate_index(idx + 3);
        f32::from_le_bytes([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ]) as f64
    }

    pub fn get_floats(&self, idx: usize, n: usize) -> Vec<f64> {
        /*
            Read an f32 from the buffer.
            return as f64 just because.
        */
        self.data[idx..idx + (4 * n)]
            .chunks_exact(4)
            .map(|x| f32::from_le_bytes(x.try_into().unwrap()) as f64)
            .collect()
    }

    pub fn put_float(&mut self, idx: usize, value: f64) {
        /*
            Write an f64 to the buffer.
            actually writes as f32
        */
        self.puts(idx, (value as f32).to_le_bytes().to_vec());
    }

    pub fn put_floats(&mut self, idx: usize, values: Vec<f64>) {
        /*
            Write an f64 to the buffer.
            actually writes as f32
        */
        values
            .iter()
            .enumerate()
            .for_each(|(i, v)| self.put_float((4 * i as usize) + idx, *v));
    }

    pub fn gets(&self, idx: usize, n: usize) -> Vec<u8> {
        /*
            Read a vec of values from the buffer.
        */
        self.validate_index(idx);
        self.validate_index(idx + n - 1);
        self.data[idx..idx + n].to_vec()
    }

    pub fn puts(&mut self, idx: usize, data: Vec<u8>) {
        /*
            Write a vec of values to the buffer.
        */
        self.validate_index(idx);
        self.validate_index(idx + data.len() - 1);
        self.data[idx..idx + data.len()].copy_from_slice(&data);
    }

    pub fn reset(&mut self, n: usize) {
        /*
            Reset the buffer.
        */
        self.data = vec![0u8; n];
        self.update_flag = false;
        self.timestamp = Instant::now();
    }

    pub fn print(&self) {
        /*
            Print the buffer.
        */
        let mut data_string: String = "\n\n==========\n".to_string();

        self.data.iter().enumerate().for_each(|(i, u)| {
            data_string.push_str(format!("{:#X}\t", u).as_str());

            if (i + 1) % 16 == 0 && i != 0 {
                data_string.push_str("\n");
            }
        });

        println!("{}", data_string);
    }
}
