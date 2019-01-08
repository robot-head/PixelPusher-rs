use std::net::Ipv4Addr;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use hwaddr::HwAddr;

/**
 * Device Header format:
 * uint8_t mac_address[6];
 * uint8_t ip_address[4];
 * uint8_t device_type;
 * uint8_t protocol_version; // for the device, not the discovery
 * uint16_t vendor_id;
 * uint16_t product_id;
 * uint16_t hw_revision;
 * uint16_t sw_revision;
 * uint32_t link_speed; // in bits per second
 */
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

pub fn parse_from(buf: [u8; 84]) -> DeviceHeader {
    let hw_addr = HwAddr::from(&buf[0..6]);
    let mut rdr = Cursor::new(&buf[..]);
    rdr.set_position(6);
    let ipu32 = rdr.read_u32::<BigEndian>().unwrap();
    let ip_addr = Ipv4Addr::from(ipu32);
    let device_type_u8 = rdr.read_u8().unwrap();
    let device_type = match device_type_u8 {
        0 => DeviceType::ETHERDREAM,
        1 => DeviceType::LUMIABRIDGE,
        2 => DeviceType::PIXELPUSHER,
        _ => DeviceType::UNKNOWN,
    };
    let protocol_version = rdr.read_u8().unwrap();
    let vendor_id = rdr.read_u16::<BigEndian>().unwrap();
    let product_id = rdr.read_u16::<BigEndian>().unwrap();
    let hw_revision = rdr.read_u16::<BigEndian>().unwrap();
    let sw_revision = rdr.read_u16::<BigEndian>().unwrap();
    let link_speed = rdr.read_u32::<BigEndian>().unwrap();
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
    return device_header;
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use super::*;


    #[test]
    fn test_can_parse_live_discovery() {
        let socket = UdpSocket::bind("0.0.0.0:7331").unwrap();
        let mut buf = [0; 84];
        let (amt, _) = socket.recv_from(&mut buf).unwrap();
        assert_eq!(amt, 84);
        let device_header = parse_from(buf);
        assert_eq!(device_header.device_type, DeviceType::PIXELPUSHER);
    }
}