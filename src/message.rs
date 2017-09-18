use serde;
use serde_json;
use serde_derive;

use serde_json::Error;

use std::time::{SystemTime, UNIX_EPOCH};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::net::TcpStream;
use std::io::{Write, Read};
use std::str;
use std::fmt::Debug;


pub fn serialize_u16(val: u16) -> Vec<u8> {
    let mut buf = [0; 2];
    LittleEndian::write_u16(&mut buf, val);
    return buf.to_vec();
}

pub fn serialize_u32(val: u32) -> Vec<u8> {
    let mut buf = [0; 4];
    LittleEndian::write_u32(&mut buf, val);
    return buf.to_vec();
}

pub fn serialize_i32(val: i32) -> Vec<u8> {
    let mut buf = [0; 4];
    LittleEndian::write_i32(&mut buf, val);
    return buf.to_vec();
}


pub fn deserialize_u16(buf: &[u8]) -> u16 {
    LittleEndian::read_u16(buf)
}

pub fn deserialize_u32(buf: &[u8]) -> u32 {
    LittleEndian::read_u32(buf)
}

pub fn deserialize_i32(buf: &[u8]) -> i32 {
    LittleEndian::read_i32(buf)
}

pub trait SnapMessageData: Debug {
    fn serialize_vec(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self where Self: Sized;
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Base = 0,
    CodecHeader = 1,
    WireChunk = 2,
    ServerSettings = 3,
    Time = 4,
    Hello = 5,
}

impl From<u8> for MessageType {
    fn from(t:u8) -> MessageType {
        match t {
            0 => MessageType::Base,
            1 => MessageType::CodecHeader,
            2 => MessageType::WireChunk,
            3 => MessageType::ServerSettings,
            4 => MessageType::Time,
            5 => MessageType::Hello,
            _ => panic!("{:?} not in type-range (0-5)", t)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeVal {
    pub sec: i32,
    pub usec: i32
}

impl TimeVal {
    fn serialize(&self) -> Vec<u8> {
        let mut tv_vec = Vec::new();
        tv_vec.extend(serialize_i32(self.sec));
        tv_vec.extend(serialize_i32(self.usec));
        return tv_vec
    }

    pub fn new() -> TimeVal {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        TimeVal {
            sec: since_the_epoch.as_secs() as i32,
            usec: ((since_the_epoch.subsec_nanos() as f64)*1000.0) as i32
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub type_: MessageType,
    pub id: u16,
    pub refers_to: u16,
    pub recieved: TimeVal,
    pub sent: TimeVal,
    pub data: Box<SnapMessageData>
}

const BASE_MESSAGE_SIZE: usize = 26;

#[derive(Debug, Clone)]
pub struct BaseData {
}

impl SnapMessageData for BaseData {
    fn serialize_vec(&self) -> Vec<u8> {
        Vec::new()
    }
    fn deserialize(data: &[u8]) -> Self {
        BaseData {}

    }
}

impl Message {

    pub fn serialize(&self) -> Vec<u8> {
        let mut msg_vec = Vec::new();
        //println!("type: {:?}, {:?}", self.type_, self.type_.to_type_int());
        let type_int = self.type_.clone() as u16;
        msg_vec.extend(serialize_u16(type_int));
        msg_vec.extend(serialize_u16(self.id));
        msg_vec.extend(serialize_u16(self.refers_to));
        msg_vec.extend(self.recieved.serialize());
        msg_vec.extend(self.sent.serialize());
        let data_seialized = self.data.serialize_vec();
        msg_vec.extend(serialize_u32(data_seialized.len() as u32));
        msg_vec.extend(data_seialized.clone());
        return msg_vec
    }

    pub fn deserialize(data: Vec<u8>) {

    }

    pub fn deserialize_from_socket(mut socket: TcpStream) -> Message {

        //let r_ = socket.read(&mut buf);
        //println!("Buffer: {:?}", buf);
        let type_ = socket.read_u16::<LittleEndian>().unwrap();
        println!("Message Type: {:?}", type_);
        let id = socket.read_u16::<LittleEndian>().unwrap();
        println!("ID: {:?}", id);
        let refers_to = socket.read_u16::<LittleEndian>().unwrap();
        println!("RefersTo: {:?}", refers_to);
        let recv_sec = socket.read_i32::<LittleEndian>().unwrap();
        let recv_usec = socket.read_i32::<LittleEndian>().unwrap();
        println!("Recieved: ({:?}, {:?})", recv_sec, recv_usec);
        let sent_sec = socket.read_i32::<LittleEndian>().unwrap();
        let sent_usec = socket.read_i32::<LittleEndian>().unwrap();
        println!("Sent: ({:?}, {:?})", sent_sec, sent_usec);
        let data_size = socket.read_u32::<LittleEndian>().unwrap();
        println!("Size: {:?}", data_size);
        let mut buf = vec![0; data_size as usize];
        let data = socket.read(&mut buf);
        let deserialized_data_: Box<SnapMessageData> = match type_ {
            //1 => MessageType::CodecHeader,
            //2 => MessageType::WireChunk,
            3 => Box::new(ServerSettingsData::deserialize(&buf)),
            //4 => MessageType::Time,
            5 => Box::new(HelloData::deserialize(&buf)),
            _ => Box::new(BaseData {}),
        };
        let deserialized_data = deserialized_data_;
        Message {
            type_: MessageType::from(type_ as u8),
            id: id,
            refers_to: refers_to,
            recieved: TimeVal { sec: recv_sec, usec: recv_usec },
            sent: TimeVal { sec: sent_sec, usec: sent_usec },
            data: deserialized_data
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloData {
    #[serde(rename = "MAC")]
    pub mac: String,
    #[serde(rename = "HostName")]
    pub hostname: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "ClientName")]
    pub client_name: String,
    #[serde(rename = "OS")]
    pub os: String,
    #[serde(rename = "Arch")]
    pub arch: String,
    #[serde(rename = "Instance")]
    pub instance: usize,
    #[serde(rename = "SnapStreamProtocolVersion")]
    pub snap_stream_protocol_version: usize
}

impl SnapMessageData for HelloData {
    fn serialize_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        let s: String = serde_json::to_string(&self).unwrap();
        let s = s.as_bytes().to_vec();
        v.extend(serialize_u32(s.len() as u32));
        v.extend(s);
        v
    }
    fn deserialize(data: &[u8]) -> HelloData {
        let s: HelloData = serde_json::from_slice(&data[4..]).unwrap();
        s
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettingsData {
    pub muted: bool,
    #[serde(rename = "bufferMs")]
    pub buffer_ms: i32,
    pub latency: i32,
    pub volume: u16
}

impl SnapMessageData for ServerSettingsData {
    fn serialize_vec(&self) -> Vec<u8> {
        Vec::new()
    }
    fn deserialize(data: &[u8]) -> ServerSettingsData {
        let s = serde_json::from_slice(&data[4..]).unwrap();
        s

    }
}
