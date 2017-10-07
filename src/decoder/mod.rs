use message;

mod pcm_decoder;
pub use self::pcm_decoder::PCMDecoder;

use alsa::pcm::{PCM, HwParams, Format, Access, State};

pub trait Decoder {
    fn decode(&self, chunk: Vec<u8>) -> Vec<i16>;
    fn setHeader(&self, header: message::CodecHeaderData);
    fn new() -> Self where Self: Sized;
    fn get_hwparams(&self, pcm: &HwParams);
}

pub struct DummyDecoder;

impl Decoder for DummyDecoder {
    fn decode(&self, chunk: Vec<u8>) -> Vec<i16> {
        Vec::new()
    }
    fn setHeader(&self, header: message::CodecHeaderData) {

    }
    fn new() -> Self {
        Self {}
    }
    fn get_hwparams(&self, pcm: &HwParams) { }
}

