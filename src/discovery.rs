use std::net::Ipv4Addr;
use std::io::Cursor;
use std::time::Duration;
use byteorder::{LittleEndian, ReadBytesExt};

use hwaddr::HwAddr;

use std::vec::Vec;
use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::SystemTime;
use std::collections::HashSet;


#[derive(Debug, PartialEq)]
pub enum DeviceType {
    ETHERDREAM,
    LUMIABRIDGE,
    PIXELPUSHER,
    UNKNOWN,
}

#[derive(Debug)]
pub struct DeviceHeader {
    mac_addr: HwAddr,
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
    device_header: DeviceHeader,
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
    DeviceHeader(DeviceHeader),
    PixelPusherHeader(PixelPusherHeader),
}

impl Header {
    fn parse(buf: [u8; 84]) -> Header {
        let hw_addr = HwAddr::from(&buf[0..6]);
        let mut rdr = Cursor::new(&buf[..]);
        rdr.set_position(6);
        let ipu32 = rdr.read_u32::<LittleEndian>().unwrap();
        let ip_addr = Ipv4Addr::from(ipu32);
        let device_type_u8 = rdr.read_u8().unwrap();
        let device_type = match device_type_u8 {
            0 => DeviceType::ETHERDREAM,
            1 => DeviceType::LUMIABRIDGE,
            2 => DeviceType::PIXELPUSHER,
            _ => DeviceType::UNKNOWN,
        };
        let protocol_version = rdr.read_u8().unwrap();
        let vendor_id = rdr.read_u16::<LittleEndian>().unwrap();
        let product_id = rdr.read_u16::<LittleEndian>().unwrap();
        let hw_revision = rdr.read_u16::<LittleEndian>().unwrap();
        let sw_revision = rdr.read_u16::<LittleEndian>().unwrap();
        let link_speed = rdr.read_u32::<LittleEndian>().unwrap();
        let device_header = DeviceHeader {
            mac_addr: hw_addr,
            ip_addr: ip_addr,
            device_type: device_type,
            protocol_version: protocol_version,
            vendor_id: vendor_id,
            product_id: product_id,
            hw_revision: hw_revision,
            sw_revision: sw_revision,
            link_speed: link_speed,
        };
        match device_header.device_type {
            DeviceType::PIXELPUSHER => {
                let strips_attached = rdr.read_u8().unwrap();
                let max_strips_per_packet = rdr.read_u8().unwrap();
                let pixels_per_strip = rdr.read_u16::<LittleEndian>().unwrap();
                let update_period = rdr.read_u32::<LittleEndian>().unwrap();
                let power_total = rdr.read_u32::<LittleEndian>().unwrap();
                let delta_sequence = rdr.read_u32::<LittleEndian>().unwrap();
                let controller = rdr.read_u32::<LittleEndian>().unwrap();
                let group = rdr.read_u32::<LittleEndian>().unwrap();
                let artnet_universe = rdr.read_u16::<LittleEndian>().unwrap();
                let artnet_channel = rdr.read_u16::<LittleEndian>().unwrap();
                let my_port = rdr.read_u16::<LittleEndian>().unwrap();
                let pusher_header = PixelPusherHeader {
                    device_header: device_header,
                    strips_attached: strips_attached,
                    max_strips_per_packet: max_strips_per_packet,
                    pixels_per_strip: pixels_per_strip,
                    update_period: update_period,
                    power_total: power_total,
                    delta_sequence: delta_sequence,
                    controller: controller,
                    group: group,
                    artnet_universe: artnet_universe,
                    artnet_channel: artnet_channel,
                    my_port: my_port,
                };
                return Header::PixelPusherHeader(pusher_header);
            }
            _ => {
                return Header::DeviceHeader(device_header);
            }
        }
    }
}

pub fn discover(timeout_secs: u64) -> Vec<Header> {
    let (tx_headers, rx_headers) = channel();

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
            break;
        }
        let header = rx_headers.recv_timeout(Duration::from_secs(timeout_secs));
        if header.is_ok() {
            let val = header.unwrap();
            let mut mac_addr: String;
            match &val {
                Header::PixelPusherHeader(pusher_header) => {
                    mac_addr = pusher_header.device_header.mac_addr.to_string();
                }
                Header::DeviceHeader(device_header) => {
                    mac_addr = device_header.mac_addr.to_string();
                }
            }
            if seen_macs.contains(&mac_addr) {
                continue;
            }
            seen_macs.insert(mac_addr);
            headers.push(val);
        } else {
            break;
        }
    }

    return headers;
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use super::*;


    #[test]
    fn test_discover() {
        let headers = discover(3);
        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn test_can_parse_live_discovery() {
        let socket = UdpSocket::bind("0.0.0.0:7331").unwrap();
        let mut buf = [0; 84];
        let (amt, _) = socket.recv_from(&mut buf).unwrap();
        assert_eq!(amt, 84);
        let device_header = Header::parse(buf);

        match device_header {
            Header::PixelPusherHeader(_header) => {
                // TODO
            }
            _ => {
                panic!("Expected to find a PixelPusher")
            }
        }
    }
}