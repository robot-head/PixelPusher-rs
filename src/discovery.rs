use std::collections::HashSet;
use std::io::Error;
use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::vec::Vec;

use log::{info, trace, warn};

use crate::device::Header;

#[derive(Debug, Clone)]
struct ParseError;

pub fn discover(timeout_secs: u64) -> Option<Vec<Header>> {
    let (tx_headers, rx_headers) = channel();

    trace!("Spawn discovery listener thread");
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:7331").unwrap();
        let mut buf = [0; 84];
        loop {
            let (_amt, _snd) = socket.recv_from(&mut buf).unwrap();
            tx_headers.send(Header::parse(buf));
        }
    });

    let mut headers: Vec<Header> = Vec::new();
    let start = SystemTime::now();
    let mut seen_macs = HashSet::new();
    loop {
        if (start + Duration::from_secs(timeout_secs)) < SystemTime::now() {
            trace!("Discovery timeout ended");
            break;
        }
        let header = rx_headers.recv_timeout(Duration::from_secs(timeout_secs));
        if header.is_ok() {
            let parse_result: Result<Header, Error> = header.unwrap();
            if parse_result.is_err() {
                warn!("{}", parse_result.unwrap_err());
                continue;
            }
            let val = parse_result.unwrap();
            let mut mac_addr: String;

            match &val {
                Header::PixelPusherHeader(pusher_header) => {
                    mac_addr = pusher_header.base_header.hw_addr.to_string();
                }
            }
            if seen_macs.contains(&mac_addr) {
                trace!("Seen device already at addr {}", mac_addr);
                continue;
            }
            seen_macs.insert(mac_addr);
            headers.push(val);
        } else {
            warn!("{}", header.unwrap_err());
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