use std::collections::HashSet;
use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::vec::Vec;

use log::{info, trace, warn};

use crate::device::DeviceHeader;
use crate::device::DeviceType;

#[derive(Debug, Clone)]
struct ParseError;

pub fn discover(timeout_secs: u64) -> Vec<Box<dyn DeviceHeader>> {
    let (tx_headers, rx_headers) = channel();

    trace!("Spawn discovery listener thread");
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:7331").unwrap();
        let mut buf = [0; 84];
        loop {
            let (_amt, _snd) = socket.recv_from(&mut buf).unwrap();
            let dh = crate::device::parse_header(buf);
            tx_headers.send(dh);
        }
    });

    let mut headers: Vec<Box<dyn DeviceHeader>> = Vec::new();
    let start = SystemTime::now();
    let mut seen_macs = HashSet::new();
    loop {
        if (start + Duration::from_secs(timeout_secs)) < SystemTime::now() {
            trace!("Discovery timeout ended");
            break;
        }
        let header = rx_headers.recv_timeout(Duration::from_secs(timeout_secs));
        if header.is_ok() {
            let val = header.unwrap();
            let mac_addr = val.hw_addr();
            if seen_macs.contains(&mac_addr) {
                trace!("Seen device already at addr {}", mac_addr);
                continue;
            }
            seen_macs.insert(mac_addr);
            headers.push(val);
        } else {
            warn!("Timeout reached");
        }
    }
    headers
}

pub fn discover_type(timeout_secs: u64, device_type: DeviceType) -> Vec<Box<dyn DeviceHeader>> {
    let all_devices = discover(timeout_secs);
    let mut filtered_devices: Vec<Box<DeviceHeader>> = Vec::new();
    for device in all_devices {
        if device.device_type() == device_type {
            filtered_devices.push(device);
        }
    }
    filtered_devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover() {
        let headers = discover(3);
        assert_eq!(headers.is_some());
    }
}