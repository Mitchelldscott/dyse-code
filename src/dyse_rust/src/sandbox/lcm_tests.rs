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
// use std::{
// 	env,
// 	time::{Instant, Duration},
// 	net::{UdpSocket, SocketAddr, Ipv4Addr, IpAddr},
// };

// use lcm::Lcm;

// #[cfg(test)]
// pub mod lcm_init {
// 	use super::*;
// 	pub fn lcm_init() {
// 		let mut lcm = Lcm::new().unwrap();
// 		lcm.subscribe("Playboy", |_: String| println!("Hey there, playboy!"));
// 		lcm.publish("Playboy", &"Charles".to_string()).unwrap();
// 	}
// }
