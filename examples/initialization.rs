use std::io::BufRead;
use std::sync::{Arc, Mutex};

use vigem_client::*;

fn main() {
    let client = Client::new().unwrap();
    let mut client = Arc::new(Mutex::new(client));
    let device = client.plug_in().unwrap();

    std::io::stdin().lock().lines().next().unwrap().unwrap();

    device.unplug().unwrap();
}
