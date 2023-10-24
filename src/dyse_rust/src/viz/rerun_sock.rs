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
    socks::socks::*,
    viz::recorder::*,
};
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::Builder,
};

fn main() {
    let mut names: Vec<String> = env::args().collect();
    names.remove(0);

    let rsock = ReRunSockage::new(names);
    let store_info = rerun::new_store_info(env::var("ROBOT_NAME"));

    rerun::native_viewer::spawn(store_info, Default::default(), |rec| {
        rsock.run(&rec).unwrap();
    })?;

    Ok(())
}