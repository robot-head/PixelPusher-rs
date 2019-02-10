use std::collections::HashSet;
use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::vec::Vec;

use byteorder::{LittleEndian, ReadBytesExt};
use hwaddr::HwAddr;
use log::{info, trace, warn};

#[derive(Debug, Clone)]
struct ParseError;


#[derive(Debug, PartialEq)]
pub enum DeviceType {
    ETHERDREAM,
    LUMIABRIDGE,
    PIXELPUSHER,
    UNKNOWN,
}

#[derive(Debug)]
pub struct BaseHeader {
    hw_addr: HwAddr,
    ip_addr: Ipv4Addr,
    device_type: DeviceType,
    protocol_version: u8,
    vendor_id: u16,
    product_id: u16,
    hw_revision: u16,
    sw_revision: u16,
    link_speed: u32,
}

#[derive(Debug)]
pub struct PixelPusherHeader {
    base_header: BaseHeader,
    strips_attached: u8,
    max_strips_per_packet: u8,
    pixels_per_strip: u16,
    update_period: u32,
    power_total: u32,
    delta_sequence: u32,
    controller: u32,
    group: u32,
    artnet_universe: u16,
    artnet_channel: u16,
    my_port: u16,
}

#[derive(Debug)]
pub enum Header {
    PixelPusherHeader(PixelPusherHeader),
}

impl Header {
    fn parse(buf: [u8; 84]) -> Result<Header, std::io::Error> {
        let hw_addr = HwAddr::from(&buf[0..6]);
        let mut rdr = Cursor::new(&buf[..]);
        rdr.set_position(6);
        let ipu32 = rdr.read_u32::<LittleEndian>()?;
        let ip_addr = Ipv4Addr::from(ipu32);
        let device_type_u8 = rdr.read_u8()?;
        let device_type = match device_type_u8 {
            0 => DeviceType::ETHERDREAM,
            1 => DeviceType::LUMIABRIDGE,
            2 => DeviceType::PIXELPUSHER,
            _ => DeviceType::UNKNOWN,
        };
        let protocol_version = rdr.read_u8()?;
        let vendor_id = rdr.read_u16::<LittleEndian>()?;
        let product_id = rdr.read_u16::<LittleEndian>()?;
        let hw_revision = rdr.read_u16::<LittleEndian>()?;
        let sw_revision = rdr.read_u16::<LittleEndian>()?;
        let link_speed = rdr.read_u32::<LittleEndian>()?;
        let base_header = BaseHeader {
            hw_addr,
            ip_addr,
            device_type,
            protocol_version,
            vendor_id,
            product_id,
            hw_revision,
            sw_revision,
            link_speed,
        };
        match base_header.device_type {
            DeviceType::PIXELPUSHER => {
                let strips_attached = rdr.read_u8()?;
                let max_strips_per_packet = rdr.read_u8()?;
                let pixels_per_strip = rdr.read_u16::<LittleEndian>()?;
                let update_period = rdr.read_u32::<LittleEndian>()?;
                let power_total = rdr.read_u32::<LittleEndian>()?;
                let delta_sequence = rdr.read_u32::<LittleEndian>()?;
                let controller = rdr.read_u32::<LittleEndian>()?;
                let group = rdr.read_u32::<LittleEndian>()?;
                let artnet_universe = rdr.read_u16::<LittleEndian>()?;
                let artnet_channel = rdr.read_u16::<LittleEndian>()?;
                let my_port = rdr.read_u16::<LittleEndian>()?;
                let pusher_header = PixelPusherHeader {
                    base_header,
                    strips_attached,
                    max_strips_per_packet,
                    pixels_per_strip,
                    update_period,
                    power_total,
                    delta_sequence,
                    controller,
                    group,
                    artnet_universe,
                    artnet_channel,
                    my_port,
                };
                Ok(Header::PixelPusherHeader(pusher_header))
            }
            _ => {
                Err(Error::new(ErrorKind::Other, "Unrecognized device type"))
            }
        }
    }
}

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
        ()
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