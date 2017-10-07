use serde;
use serde_json;
use serde_derive;

use std::time::{SystemTime, UNIX_EPOCH};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::net::TcpStream;
use std::io::{Write, Read, Error};
use std::str;
use std::fmt::Debug;
use std::marker::Send;

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

pub trait SnapMessageData: Debug + Send {
    fn serialize_vec(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self where Self: Sized;
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Base(BaseData),
    CodecHeader(CodecHeaderData),
    WireChunk(WireChunkData),
    ServerSettings(ServerSettingsData),
    Time(TimeData),
    Hello(HelloData),
}

/*impl From<u8> for MessageType {
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
}*/

impl MessageType {
    fn into_int(&self) -> u16 {
        match self {
            &MessageType::Base(_) => 0,
            &MessageType::CodecHeader(_) => 1,
            &MessageType::WireChunk(_) => 2,
            &MessageType::ServerSettings(_) => 3,
            &MessageType::Time(_) => 4,
            &MessageType::Hello(_) => 5
        }
    }
}

impl MessageType {
    fn serialize_vec(&self) -> Vec<u8> {
        let t: &SnapMessageData = match self {
            &MessageType::Base(ref e) => e,
            &MessageType::CodecHeader(ref e) => e,
            &MessageType::WireChunk(ref e) => e,
            &MessageType::ServerSettings(ref e) => e,
            &MessageType::Time(ref e) => e,
            &MessageType::Hello(ref e) => e,
        };
        t.serialize_vec()
    }
}

/*impl MessageType {
    fn serialize_vec(&self) -> Vec<u8> {
        let t: Box<&SnapMessageData> = match self {
            &MessageType::Base(ref e) => Box::new(e),
            &MessageType::CodecHeader(ref e) => Box::new(e),
            &MessageType::WireChunk(ref e) => Box::new(e),
            &MessageType::ServerSettings(ref e) => Box::new(e),
            &MessageType::Time(ref e) => Box::new(e),
            &MessageType::Hello(ref e) => Box::new(e),
        };
        t.serialize_vec()
    }
}*/


#[derive(Debug, Clone)]
pub struct TimeVal {
    pub sec: isize,
    pub usec: isize
}

impl TimeVal {
    fn serialize(&self) -> Vec<u8> {
        let mut tv_vec = Vec::new();
        tv_vec.extend(serialize_i32(self.sec as i32));
        tv_vec.extend(serialize_i32(self.usec as i32));
        return tv_vec
    }

    pub fn new() -> TimeVal {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let tv = TimeVal {
            sec: since_the_epoch.as_secs() as isize,
            usec: (since_the_epoch.subsec_nanos()/1000) as isize
        };
        tv
    }
}

#[derive(Debug)]
pub struct Message {
    pub type_: MessageType,
    pub id: u16,
    pub refers_to: u16,
    pub recieved: TimeVal,
    pub sent: TimeVal,
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
        let type_int = self.type_.into_int();
        msg_vec.extend(serialize_u16(type_int));
        msg_vec.extend(serialize_u16(self.id));
        msg_vec.extend(serialize_u16(self.refers_to));
        msg_vec.extend(self.recieved.serialize());
        msg_vec.extend(self.sent.serialize());
        let data_serialized = self.type_.serialize_vec();
        msg_vec.extend(serialize_u32(data_serialized.len() as u32));
        msg_vec.extend(data_serialized.clone());
        return msg_vec
    }

    pub fn deserialize(data: Vec<u8>) {

    }

    pub fn deserialize_from_socket(mut socket: &TcpStream) -> Result<Message, Error> {
        let type_ = socket.read_u16::<LittleEndian>()?;
        debug!("Message Type: {:?}", type_);
        let id = socket.read_u16::<LittleEndian>()?;
        debug!("ID: {:?}", id);
        let refers_to = try!(socket.read_u16::<LittleEndian>());
        debug!("RefersTo: {:?}", refers_to);
        let recv_sec = socket.read_i32::<LittleEndian>()?;
        let recv_usec = socket.read_i32::<LittleEndian>()?;
        debug!("Recieved: ({:?}, {:?})", recv_sec, recv_usec);
        let sent_sec = socket.read_i32::<LittleEndian>()?;
        let sent_usec = socket.read_i32::<LittleEndian>()?;
        debug!("Sent: ({:?}, {:?})", sent_sec, sent_usec);
        let data_size = socket.read_u32::<LittleEndian>()?;
        debug!("Size: {:?}", data_size);
        let mut buf = vec![0; data_size as usize];
        let data = socket.read(&mut buf);
        let type_: MessageType = match type_ {
            1 => MessageType::CodecHeader(CodecHeaderData::deserialize(&buf)),
            2 => MessageType::WireChunk(WireChunkData::deserialize(&buf)),
            3 => MessageType::ServerSettings(ServerSettingsData::deserialize(&buf)),
            4 => MessageType::Time(TimeData::deserialize(&buf)),
            5 => MessageType::Hello(HelloData::deserialize(&buf)),
            _ => MessageType::Base(BaseData {}),
        };
        Ok(Message {
            type_: type_,
            id: id,
            refers_to: refers_to,
            recieved: TimeVal { sec: recv_sec as isize, usec: recv_usec as isize },
            sent: TimeVal { sec: sent_sec as isize, usec: sent_usec as isize },
        })
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

#[derive(Debug, Clone)]
pub struct TimeData {
    pub latency: TimeVal
}

impl SnapMessageData for TimeData {
    fn serialize_vec(&self) -> Vec<u8> {
        let v = Vec::new();
        v
    }
    fn deserialize(data: &[u8]) -> Self {
        let sec = LittleEndian::read_i32(&data[..4]);
        let usec = LittleEndian::read_i32(&data[4..]);
        TimeData {
            latency: TimeVal {
                sec: sec as isize,
                usec: usec as isize
            }
        }

    }
}

#[derive(Debug, Clone)]
pub struct CodecHeaderData {
    pub codec: String,
    pub payload: Vec<u8>
}

impl SnapMessageData for CodecHeaderData {
    fn serialize_vec(&self) -> Vec<u8> {
        let v = Vec::new();
        v
    }

    fn deserialize(data: &[u8]) -> CodecHeaderData {
        let mut data = Vec::from(data);
        let (codec_len_, data) = data.split_at(4);
        let codec_len = LittleEndian::read_u32(codec_len_);
        let (codec_, data) = data.split_at(codec_len as usize);
        let codec = String::from_utf8(codec_.to_vec()).unwrap();
        let (payload_len_, data) = data.split_at(4);
        let payload_len = LittleEndian::read_u32(payload_len_);
        let (payload, data) = data.split_at(payload_len as usize);
        assert!(data.len() == 0);
        CodecHeaderData {
            codec: codec,
            payload: payload.to_vec()
        }
    }
}

#[derive(Debug, Clone)]
pub struct WireChunkData {
    pub timestamp: TimeVal,
    pub payload: Vec<u8>
}
impl SnapMessageData for WireChunkData {
    fn serialize_vec(&self) -> Vec<u8> {
        let v = Vec::new();
        v
    }

    fn deserialize(data: &[u8]) -> WireChunkData {
        let mut data = Vec::from(data);
        let (sec_, data) = data.split_at(4);
        let (usec_, data) = data.split_at(4);
        let (payload_len_, data) = data.split_at(4);
        let sec = LittleEndian::read_i32(sec_);
        let usec = LittleEndian::read_i32(usec_);
        let payload_len = LittleEndian::read_i32(payload_len_);
        let (payload, data) = data.split_at(payload_len as usize);
        assert!(data.len() == 0);
        WireChunkData {
            payload: payload.to_vec(),
            timestamp: TimeVal {
                sec: sec as isize,
                usec: usec as isize
            }
        }
    }
}
