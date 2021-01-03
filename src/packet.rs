use std::error::Error;
use std::io::{Cursor,Write,Read};
use byteorder::{WriteBytesExt, ReadBytesExt,BigEndian};
use std::fmt;
use crate::processable::Processable;

const U32_U10_MASK:u32 = 0b00000011_11111111u16 as u32;
const U32_U9_MASK:u32 = 0b00000001_11111111u16 as u32;
const U32_U4_MASK:u32 = 0b00001111u8 as u32;

pub struct OuterPacket {
    pub data_packet: bool,
    pub pid: u16,
    pub cid: u16,
    pub iid: u8,
    pub data:Vec<u8>,
}

impl fmt::Display for OuterPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ data_packet:{}, pid:{}, cid:{}, iid:{}, data:{:#04X?} }}", self.data_packet, self.pid, self.cid, self.iid, self.data)
    }
}

impl Processable for OuterPacket {
    fn decode(d:&[u8]) -> Result<Self, Box<dyn Error>> {
        let mut rdr = Cursor::new(d);
        let header = rdr.read_u24::<BigEndian>().unwrap();
        let data_packet:bool = (header >> 23) & 0b1 == 0b1;
        let pid:u16 = ((header >> 13) & U32_U10_MASK)  as u16; 
        let iid:u8 = ((header >> 9) & U32_U4_MASK)  as u8;
        let cid:u16 = (header & U32_U9_MASK) as u16;
        let mut data = Vec::new();
        rdr.read_to_end(&mut data)?;
        return Ok(OuterPacket {
            data_packet,
            pid,
            iid,
            cid,
            data
        });
    }
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut wtr = vec![];
        let data_packet:u32 = (self.data_packet as u32) << 23;
        let pid:u32 = ((self.pid as u32) & U32_U10_MASK) << 13;
        let iid:u32 =  ((self.iid as u32) & U32_U4_MASK) << 9;
        let cid:u32 =  (self.cid as u32) & U32_U9_MASK;
        wtr.write_u24::<BigEndian>(data_packet | pid | iid | cid)?;
        wtr.write_all(&self.data)?;
        return Ok(wtr);
    }
}