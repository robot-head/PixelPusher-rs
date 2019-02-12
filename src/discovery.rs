use std::collections::HashSet;
use std::io::Error;
use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::vec::Vec;

use hwaddr::HwAddr;
use log::{info, trace, warn};

use crate::device::DeviceHeader;
use crate::device::parse_header;

#[derive(Debug, Clone)]
struct ParseError;

pub fn discover(timeout_secs: u64) -> Option<Vec<Box<DeviceHeader>>> {
    let (tx_headers, rx_headers) = channel();

    trace!("Spawn discovery listener thread");
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:7331").unwrap();
        let mut buf = [0; 84];
        loop {
            let (_amt, _snd) = socket.recv_from(&mut buf).unwrap();
            let dh = parse_header(buf);
            tx_headers.send(dh);
        }
    });

    let mut headers: Vec<Box<DeviceHeader>> = Vec::new();
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
            headers.push(Box::from(val));
        } else {
            warn!("Timeout reached");
        }
    }
    if headers.is_empty() {
        // Return none
    }
    Some(headers)
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