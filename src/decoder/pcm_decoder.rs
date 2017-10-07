use message;
use decoder::Decoder;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::ops::Range;
use hound;
use std::io::Cursor;
use std::mem;

use alsa::pcm::{PCM, HwParams, Format, Access, State};
use alsa::{Direction, ValueOr};

#[repr(C, packed)]
#[derive(Debug, Clone)]
struct RiffHeader {
    riff_id: u32,
    riff_size: u32,
    wave_id: u32,
    id: u32,
    size: u32,
    audio_format: i16,
    num_channels: i16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: i16,
    bits_per_sample: i16,
}

pub struct PCMDecoder;

impl Decoder for PCMDecoder {
    fn decode(&self, chunk: Vec<u8>) -> Vec<i16> {

        //let mut rdr = Cursor::new(chunk.payload);
        let mut frames = vec![0i16; chunk.len()/2];
        unsafe {
            LittleEndian::read_i16_into(chunk.as_slice(), frames.as_mut_slice());
        }
        //let frames = Vec::new();
        frames
    }
    fn setHeader(&self, mut header: message::CodecHeaderData) {
        assert!(header.payload.len() >= 44);
        //mem::transmute(s4);
        /*let id = LittleEndian::read_u32(&header.payload[0..4]);
        let sz = LittleEndian::read_u32(&header.payload[4..8]);
        let wave_id = LittleEndian::read_u32(&header.payload[8..12]);*/

        let data_ptr: *const RiffHeader = unsafe { mem::transmute(header.payload.as_ptr()) };
        let data: RiffHeader = unsafe { (*data_ptr).clone() };
        println!("{:?}", data);

    }
    fn new() -> Self {
        Self {}
    }
    fn get_hwparams(&self, hwp: &HwParams) {
        hwp.set_channels(2).unwrap();
        hwp.set_rate(48000, ValueOr::Nearest).unwrap();
        hwp.set_format(Format::s16()).unwrap();
        hwp.set_access(Access::RWInterleaved).unwrap();
    }
}
