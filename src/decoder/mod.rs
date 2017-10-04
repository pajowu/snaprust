use message;
use rodio::buffer::SamplesBuffer;
use rodio::Source;
use rodio::Sample;

mod pcm_decoder;
pub use self::pcm_decoder::PCMDecoder;


pub trait Decoder {
    fn decode(&self, chunk: message::WireChunkData) -> SamplesBuffer<i16>;
    fn setHeader(&self, header: message::CodecHeaderData);
    fn new() -> Self where Self: Sized;
}

pub struct DummyDecoder;

impl Decoder for DummyDecoder {
    fn decode(&self, chunk: message::WireChunkData) -> SamplesBuffer<i16> {
        SamplesBuffer::new(1, 44100, Vec::new())
    }
    fn setHeader(&self, header: message::CodecHeaderData) {

    }
    fn new() -> Self {
        Self {}
    }
}

