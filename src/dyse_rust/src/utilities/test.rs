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

#![allow(unused_imports)]
use crate::utilities::{data_structures::*, loaders::*};
use rand::Rng;
use std::env;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::{spawn, Builder},
    time::{Duration, Instant},
};

#[cfg(test)]
pub mod byu_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    pub fn load_yaml() {
        let yaml_data = "integer: 1\nfloat: 1.0";

        let byu = BuffYamlUtil::new(yaml_data);

        byu.parse_int("integer", byu.data())
            .expect("Did not find integer");
        byu.parse_float("float", byu.data())
            .expect("Did not find float");
        // byu.parse_int("dne", byu.data()).expect("Did not find nothing");
    }

    #[test]
    pub fn load_item_yaml() {
        let yaml_data = "item1:\n  name: thing1\n  value: 10.0";

        let byu = BuffYamlUtil::new(yaml_data);

        let item1 = byu.item("item1").expect("Didn't find item1");

        byu.parse_str("name", item1).expect("Did not find name");
        byu.parse_float("value", item1).expect("Did not find value");
    }

    #[test]
    pub fn load_items_yaml() {
        let yaml_data = "group:\n  item1:\n    name: thing1\n    value: 10.0\n  item2:\n    name: thing2\n    value: -1.0";

        let byu = BuffYamlUtil::new(yaml_data);

        let group = byu.item("group").expect("Didn't find group");

        byu.parse_str("name", &group["item1"])
            .expect("Did not find item1/name");
        byu.parse_str("name", &group["item2"])
            .expect("Did not find item2/name");
        byu.parse_float("value", &group["item1"])
            .expect("Did not find item1/value");
        byu.parse_float("value", &group["item2"])
            .expect("Did not find item2/value");
    }

    #[test]
    pub fn load_items_list_yaml() {
        /*
            Use the penguin yaml file to test some loading functions
        */
        let yaml_data = "group:\n  item1:\n    name: thing1\n    info: [10.0, 9.0, 8.0, 7.0, 6.0]\n  item2:\n    name: thing2\n    info: [10.0, 9.0, 8.0, 7.0, 6.0]";

        let byu = BuffYamlUtil::new(yaml_data);

        let group = byu.item("group").expect("Didn't find group");

        byu.parse_str("name", &group["item1"])
            .expect("Did not find item1/name");
        byu.parse_str("name", &group["item2"])
            .expect("Did not find item2/name");
        assert_eq!(
            byu.parse_floats("info", &group["item1"])
                .expect("Did not find item1/info"),
            vec![10.0, 9.0, 8.0, 7.0, 6.0],
            "wrong float list"
        );
        assert_eq!(
            byu.parse_floats("info", &group["item2"])
                .expect("Did not find item2/info"),
            vec![10.0, 9.0, 8.0, 7.0, 6.0],
            "wrong float list"
        );
    }
}

#[cfg(test)]
pub mod buffer_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    ///
    /// Test the byte buffer
    ///
    #[test]
    pub fn basic_byte_buffer() {
        let mut rng = rand::thread_rng();
        let n1: u8 = rng.gen();
        let i: usize = rng.gen_range(0..63);

        let mut buffer = ByteBuffer::new(64);
        // buffer.print_data();
        buffer.put(i, n1);
        assert_eq!(
            buffer.get(i),
            n1,
            "[{}] Failed get check {} != {}",
            i,
            n1,
            buffer.get(i)
        );
    }

    #[test]
    pub fn intermediate_byte_buffer() {
        let mut rng = rand::thread_rng();
        let n1: Vec<u8> = vec![rng.gen(); 10];
        let i: usize = rng.gen_range(0..53);

        let mut buffer = ByteBuffer::new(64);
        // buffer.print_data();
        buffer.puts(i, &n1);

        assert_eq!(
            buffer.get(i),
            n1[0],
            "[{}] Failed get check {} != {}",
            i,
            n1[0],
            buffer.get(i)
        );
        assert_eq!(
            buffer.get(i + 1),
            n1[1],
            "[{}] Failed get check {} != {}",
            i + 1,
            n1[1],
            buffer.get(i + 1)
        );
        assert_eq!(
            buffer.get(i + 2),
            n1[2],
            "[{}] Failed get check {} != {}",
            i + 2,
            n1[2],
            buffer.get(i + 2)
        );
        assert_eq!(
            buffer.get(i + 3),
            n1[3],
            "[{}] Failed get check {} != {}",
            i + 3,
            n1[3],
            buffer.get(i + 3)
        );
    }

    #[test]
    pub fn get_float_byte_buffer() {
        let n1: Vec<u8> = vec![0x40, 0x49, 0xf, 0xdb];

        let mut buffer = ByteBuffer::new(64);
        buffer.puts(2, &n1);
        buffer.print();

        assert_eq!(
            buffer.get_float(2),
            3.1415927410125732,
            "Failed Float check {} != {}",
            3.1415927410125732,
            buffer.get_float(2)
        );
    }
}
