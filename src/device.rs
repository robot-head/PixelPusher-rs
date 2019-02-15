use std::io::Cursor;
use std::net::Ipv4Addr;
use std::thread::Thread;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hwaddr::HwAddr;
use image::Rgb;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DeviceType {
    ETHERDREAM,
    LUMIABRIDGE,
    PIXELPUSHER,
    UNKNOWN,
}

pub trait DeviceHeader {
    fn hw_addr(&self) -> HwAddr;
    fn ip_addr(&self) -> Ipv4Addr;
    fn device_type(&self) -> DeviceType;
    fn serialize(&self, wtr: &Vec<u8>);
}

#[derive(Debug)]
struct Header {
    hw_addr: HwAddr,
    ip_addr: Ipv4Addr,
    pub device_type: DeviceType,
    protocol_version: u8,
    vendor_id: u16,
    product_id: u16,
    hw_revision: u16,
    sw_revision: u16,
    link_speed: u32,
}

impl DeviceHeader for Header {
    fn hw_addr(&self) -> HwAddr {
        self.hw_addr
    }

    fn ip_addr(&self) -> Ipv4Addr {
        self.ip_addr
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::UNKNOWN
    }

    fn serialize(&self, wtr: &Vec<u8>) {
        let mut w = wtr.to_owned();
        w.extend(self.hw_addr.octets().iter());
        w.extend(self.ip_addr.octets().iter());
        let device_type = match self.device_type {
            DeviceType::ETHERDREAM => 0,
            DeviceType::LUMIABRIDGE => 1,
            DeviceType::PIXELPUSHER => 2,
            DeviceType::UNKNOWN => 99,
            _ => 99
        };
        w.push(device_type);
        w.push(self.protocol_version);
        w.write_u16::<LittleEndian>(self.vendor_id).unwrap();
        w.write_u16::<LittleEndian>(self.product_id).unwrap();
        w.write_u16::<LittleEndian>(self.hw_revision).unwrap();
        w.write_u16::<LittleEndian>(self.sw_revision).unwrap();
        w.write_u32::<LittleEndian>(self.link_speed).unwrap();
    }
}

#[derive(Debug)]
struct PixelPusherHeader {
    base_header: Header,
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

impl DeviceHeader for PixelPusherHeader {
    fn hw_addr(&self) -> HwAddr {
        self.base_header.hw_addr
    }

    fn ip_addr(&self) -> Ipv4Addr {
        self.base_header.ip_addr
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::PIXELPUSHER
    }

    fn serialize(&self, wtr: &Vec<u8>) {
        self.base_header.serialize(wtr);
        let mut w = wtr.to_owned();
        w.push(self.strips_attached);
        w.push(self.max_strips_per_packet);
        w.write_u16::<LittleEndian>(self.pixels_per_strip).unwrap();
        w.write_u32::<LittleEndian>(self.update_period).unwrap();
        w.write_u32::<LittleEndian>(self.power_total).unwrap();
        w.write_u32::<LittleEndian>(self.delta_sequence).unwrap();
        w.write_u32::<LittleEndian>(self.controller).unwrap();
        w.write_u32::<LittleEndian>(self.group).unwrap();
        w.write_u16::<LittleEndian>(self.artnet_universe).unwrap();
        w.write_u16::<LittleEndian>(self.artnet_channel).unwrap();
        w.write_u16::<LittleEndian>(self.my_port).unwrap();
    }
}

pub fn parse_header(buf: [u8; 84]) -> Box<dyn DeviceHeader + Send> {
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
    let base_header = Header {
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
    match device_type {
        DeviceType::PIXELPUSHER => return Box::from(parse_pixelpusher_header(base_header, buf)),
        _ => return Box::from(base_header),
    }
}

fn parse_pixelpusher_header(base_header: Header, buf: [u8; 84]) -> PixelPusherHeader {
    let mut rdr = Cursor::new(&buf[..]);
    rdr.set_position(24);
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
    PixelPusherHeader {
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
    }
}

pub struct PixelPusher {
    header: PixelPusherHeader,
    buffer: [u8; 480 * 8 * 3],
    xmit_thread: Thread,
    update_thread: Thread,
}

impl PixelPusher {
    pub fn set_color(&mut self, strip: u8, pixel: u8, color: Rgb<u8>) {
        let x = &mut self.buffer;
        let index = (480 * 3 * (strip as usize)) + (pixel as usize * 3);
        x[index] = color.data[0];
        x[index + 1] = color.data[1];
        x[index + 2] = color.data[2];
    }
}
