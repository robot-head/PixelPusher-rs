use hwaddr::HwAddr;
use std::net::Ipv4Addr;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Error;
use std::io::ErrorKind;

#[derive(Debug, PartialEq)]
pub enum DeviceType {
    ETHERDREAM,
    LUMIABRIDGE,
    PIXELPUSHER,
    UNKNOWN,
}

#[derive(Debug)]
pub struct BaseHeader {
    pub hw_addr: HwAddr,
    pub ip_addr: Ipv4Addr,
    pub device_type: DeviceType,
    protocol_version: u8,
    vendor_id: u16,
    product_id: u16,
    hw_revision: u16,
    sw_revision: u16,
    link_speed: u32,
}

#[derive(Debug)]
pub struct PixelPusherHeader {
    pub base_header: BaseHeader,
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
    pub fn parse(buf: [u8; 84]) -> Result<Header, std::io::Error> {
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