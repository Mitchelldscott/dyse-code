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
        /*
            Use the penguin yaml file to test some loading functions
        */

        let byu = BuffYamlUtil::new("penguin");

        assert_eq!(byu.load_string("robot_type"), "demo");

        byu.load_tasks();
    }
}

#[cfg(test)]
pub mod dyseer_tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    ///
    /// Test the byte dyseer
    ///
    #[test]
    pub fn basic_byte_dyseer() {
        let mut rng = rand::thread_rng();
        let n1: u8 = rng.gen();
        let i: usize = rng.gen_range(0..63);

        let mut dyseer = ByteBuffer::new(64);
        // dyseer.print_data();
        dyseer.put(i, n1);
        assert_eq!(
            dyseer.get(i),
            n1,
            "[{}] Failed get check {} != {}",
            i,
            n1,
            dyseer.get(i)
        );
    }

    #[test]
    pub fn intermediate_byte_dyseer() {
        let mut rng = rand::thread_rng();
        let n1: Vec<u8> = vec![rng.gen(); 10];
        let i: usize = rng.gen_range(0..53);

        let mut dyseer = ByteBuffer::new(64);
        // dyseer.print_data();
        dyseer.puts(i, n1.clone());

        assert_eq!(
            dyseer.get(i),
            n1[0],
            "[{}] Failed get check {} != {}",
            i,
            n1[0],
            dyseer.get(i)
        );
        assert_eq!(
            dyseer.get(i + 1),
            n1[1],
            "[{}] Failed get check {} != {}",
            i + 1,
            n1[1],
            dyseer.get(i + 1)
        );
        assert_eq!(
            dyseer.get(i + 2),
            n1[2],
            "[{}] Failed get check {} != {}",
            i + 2,
            n1[2],
            dyseer.get(i + 2)
        );
        assert_eq!(
            dyseer.get(i + 3),
            n1[3],
            "[{}] Failed get check {} != {}",
            i + 3,
            n1[3],
            dyseer.get(i + 3)
        );
    }

    #[test]
    pub fn get_float_byte_dyseer() {
        let n1: Vec<u8> = vec![0x40, 0x49, 0xf, 0xdb];

        let mut dyseer = ByteBuffer::new(64);
        dyseer.puts(2, n1.clone());
        dyseer.print();

        assert_eq!(
            dyseer.get_float(2),
            3.1415927410125732,
            "Failed Float check {} != {}",
            3.1415927410125732,
            dyseer.get_float(2)
        );
    }
}
