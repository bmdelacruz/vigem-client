use std::io::BufRead;
use std::sync::{Arc, Mutex};

use vigem_client::*;

fn main() {
    let client = Client::new().unwrap();
    let mut client = Arc::new(Mutex::new(client));
    let mut device = client.plug_in().unwrap();

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        match line.as_str() {
            "press-x" => {
                if device.put_input(Input::Pressed(Button::X)).is_err() {
                    break;
                }
            }
            "release-x" => {
                if device.put_input(Input::Released(Button::X)).is_err() {
                    break;
                }
            }
            "" => {
                println!("latest output: {:?}", device.get_output());
            }
            "exit" => break,
            _ => {}
        }
    }

    device.unplug().unwrap();
}
