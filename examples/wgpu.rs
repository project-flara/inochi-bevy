use std::collections::HashSet;

use glutin::api::egl::device::Device;
use wgpu::{AdapterInfo, Backends, Instance};

fn main() {
    let instance = Instance::new(Backends::GL);

    let adapters: Vec<AdapterInfo> = instance
        .enumerate_adapters(Backends::GL)
        .map(|adapter| adapter.get_info())
        .collect();

    println!("{adapters:#?}");

    let devices: Vec<(Option<String>, Option<String>, HashSet<String>)> = Device::query_devices().unwrap().map(|device| {
        (
            device.name().map(|str| str.to_string()),
            device.vendor().map(|str| str.to_string()),
            device.extensions().iter().map(|str| str.to_string()).collect()
        )
    }).collect();

    println!("{devices:#?}");
    

}



