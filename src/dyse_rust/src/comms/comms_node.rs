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

use dyse_rust::comms::hid_interface::*;
use std::thread::Builder;

fn main() {
    /*
        Start an hid layer
    */
    let (mut interface, mut reader, mut writer) = HidInterface::new();

    interface.print();

    let reader_handle = Builder::new()
        .name("HID Reader".to_string())
        .spawn(move || {
            reader.pipeline();
        })
        .unwrap();

    let writer_handle = Builder::new()
        .name("HID Writer".to_string())
        .spawn(move || {
            writer.pipeline();
        })
        .unwrap();

    let interface_handle = Builder::new()
        .name("HID Control".to_string())
        .spawn(move || {
            interface.pipeline(true);
        })
        .unwrap();

    reader_handle.join().expect("HID Reader failed");
    interface_handle.join().expect("HID Control failed");
    writer_handle.join().expect("HID Writer failed");
}
