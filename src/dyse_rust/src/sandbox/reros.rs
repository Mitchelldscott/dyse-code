// /********************************************************************************
//  *
//  *      ____                     ____          __           __       _
//  *     / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
//  *    / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
//  *   / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  )
//  *  /_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/
//  *        /____/
//  *
//  *
//  *
//  ********************************************************************************/
// #![allow(unused_imports)]
// #![allow(unused_macros)]
// use rosrust_msg::std_msgs;
// use std::time::Instant;

// // macro_rules! ros_publisher {
// //     () => {{ }};
// //     ($port:expr) => {{ SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), $port) }};
// //     ($ip1:expr, $ip2:expr, $ip3:expr, $ip4:expr, $port:expr) => {{ SocketAddr::new(IpAddr::V4(Ipv4Addr::new($ip1, $ip2, $ip3, $ip4)), $port) }};
// // }

// pub enum RosPublisherType {
//     Float(rosrust::Publisher<std_msgs::Float64MultiArray>),
// }

// impl RosPublisherType {
// 	pub fn send(&self, msg: ) {

// 	}
// }

// // pub struct RosPublisherGen<T: rosrust::Message> {
// // 	publisher: rosrust::Publisher<T>,
// // }

// // impl<T: rosrust::Message> RosPublisherGen<T> {
// // 	pub fn new(name: &str) -> RosPublisherGen<T> {
// // 		RosPublisherGen {
// // 			publisher: rosrust::publish(name, 1).unwrap(),
// // 		}
// // 	}
// // }

// pub fn ros_publisher_factory(datatype: &str, name: &str) -> RosPublisherType {
// 	match datatype {
// 		_ => RosPublisherType::Float(rosrust::publish(name, 1).unwrap())
// 	}
// }

// #[cfg(test)]
// pub mod reros {
// 	use super::*;

// 	#[test]
// 	pub fn start() {
// 		// env_logger::init();
//         rosrust::init("dyse_comms");

//         let publisher = ros_publisher_factory("float", "test_topic1");

//         let t = Instant::now();
//         while t.elapsed().as_secs() < 5 {
//         	let mut msg = std_msgs::Float64MultiArray::default();
// 	        msg.data = vec![1.0, 2.0, 3.0];
// 	        publisher.send(msg);
//         }
// 	}
// }
