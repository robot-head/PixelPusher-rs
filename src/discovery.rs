use std::net::Ipv4Addr;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

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

/**
  * PixelPusher Header continues:
  * uint8_t strips_attached;
  * uint8_t max_strips_per_packet;
  * uint16_t pixels_per_strip; // uint16_t used to make alignment work
  * uint32_t update_period; // in microseconds
  * uint32_t power_total; // in PWM units
  * uint32_t delta_sequence; // difference between received and expected
  * sequence numbers
  * int32_t controller_ordinal;  // configured order number for controller
  * int32_t group_ordinal;  // configured group number for this controller
  * int16_t artnet_universe;
  * int16_t artnet_channel;
  * int16_t my_port;
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
        let device_header = Header::parse(buf);

        match device_header {
            Header::PixelPusherHeader(header) => {
                // TODO
            },
            _ => {
                panic!("Expected to find a PixelPusher")
            }
        }

    }
}