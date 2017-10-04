extern crate clap;
extern crate alsa;
extern crate byteorder;
extern crate serde;
extern crate serde_json;
extern crate hound;
extern crate rodio;


use rodio::Sink;
use rodio::buffer::SamplesBuffer;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{Config, TermLogger, WriteLogger, CombinedLogger, LogLevelFilter};


use std::ffi::CString;
use clap::{Arg, App, SubCommand};
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};
use std::net::TcpStream;
use std::io::{Write, Read};

use std::thread;
use std::sync::mpsc;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time;

mod message;
use message::SnapMessageData;

mod network_handler;
mod decoder;
use decoder::Decoder;

fn main() {
    let _ = CombinedLogger::init(
            vec![
                TermLogger::new(LogLevelFilter::Info, Config::default()).unwrap()
            ]
        );
    let matches = App::new("Snaprust Client")
    .version("0.0")
    .author("pajowu <pajowu@pajowu.de>")
    .arg(Arg::with_name("pcm_list")
        .short("l")
        .long("list")
        .help("List PCM devices"))
    .arg(Arg::with_name("HOST")
        .short("h")
        .long("host")
        .help("Sets the server hostname")
        .required(true)
        .takes_value(true))
    .arg(Arg::with_name("PORT")
        .short("p")
        .long("port")
        .help("Sets the server port")
        .required(false)
        .takes_value(true))
    .arg(Arg::with_name("CARD")
        .short("s")
        .long("soundcard")
        .help("Sets the soundcard index")
        .required(false)
        .takes_value(true))
    .get_matches();


    let host = matches.value_of("HOST").unwrap();

    if matches.is_present("pcm_list") {
        println!("all the devices");
        for x_ in alsa::card::Iter::new() {
            let x = x_.unwrap();
            debug!("{}: {} ({})", x.get_index(), x.get_name().unwrap(), x.get_longname().unwrap());
        }
    }


    //let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();


    /*println!("Connecting");
    let mut stream = TcpStream::connect("127.0.0.1:1704").unwrap();
    println!("Connected");
    let data = message::HelloData {
        arch: "x86_64".to_string(),
        client_name: "Snapclient".to_string(),
        hostname:"alpacaspacelaser".to_string(),
        instance:1,
        mac:"00:00:00:00:00:00".to_string(),
        os:"Arch Linux".to_string(),
        snap_stream_protocol_version:2,
        version:"0.11.1".to_string()};

    let hello_msg = message::Message {
        type_: message::MessageType::Hello,
        id: 0,
        refers_to: 0,
        recieved: message::TimeVal::new(),
        sent: message::TimeVal::new(),
        data: Box::new(data)
    };


    let test_hello = hello_msg.serialize();
    let test_hello: &[u8] = test_hello.as_slice();
    // ignore the Result
    stream.write_all(&test_hello[..]);
    stream.flush();
    println!("Hello Send");

    for _ in 0..50 {
        let data = message::TimeData {
            latency: message::TimeVal {
                sec: 0,
                usec: 0
            }
        };

        let time_msg = message::Message {
            type_: message::MessageType::Time,
            id: 0,
            refers_to: 0,
            recieved: message::TimeVal::new(),
            sent: message::TimeVal::new(),
            data: Box::new(data)
        };
        let msg = time_msg.serialize();
        let msg = msg.as_slice();
        stream.write_all(&msg[..]);
    }*/
    let (mut client_conn, msg_tx, msg_rx) = network_handler::ClientConnection::start("127.0.0.1:1704");

    let t = thread::spawn(move || {
        client_conn.worker();
    });

    let data = message::HelloData {
        arch: "x86_64".to_string(),
        client_name: "Snapclient".to_string(),
        hostname:"alpacaspacelaser".to_string(),
        instance:1,
        mac:"00:00:00:00:00:00".to_string(),
        os:"Arch Linux".to_string(),
        snap_stream_protocol_version:2,
        version:"0.11.1".to_string()};

    let hello_msg = message::Message {
        type_: message::MessageType::Hello(data),
        id: 0,
        refers_to: 0,
        recieved: message::TimeVal::new(),
        sent: message::TimeVal::new()
    };

    msg_tx.send(hello_msg);

    for _ in 0..50 {
        let data = message::TimeData {
            latency: message::TimeVal {
                sec: 0,
                usec: 0
            }
        };

        let time_msg = message::Message {
            type_: message::MessageType::Time(data),
            id: 0,
            refers_to: 0,
            recieved: message::TimeVal::new(),
            sent: message::TimeVal::new()
        };
        msg_tx.send(time_msg);
    }

    let endpoint = rodio::get_default_endpoint();
    if endpoint.is_none() {
        return
    }
    let endpoint = endpoint.unwrap();
    let sink = Sink::new(&endpoint);
    let mut decoder: Option<Box<Decoder>> = None;

    loop {
        while let Ok(msg) = msg_rx.try_recv() {
            debug!("Got message: {:?}", msg);
            match msg.type_ {
                message::MessageType::Base(_) => {},
                message::MessageType::CodecHeader(d) => decoder = handleCodecHeader(d),
                message::MessageType::WireChunk(d) => decoder = handleWireChunk(decoder, d, &sink),
                message::MessageType::ServerSettings(d) => handleServerSetting(d),
                message::MessageType::Time(d) => handleTime(d),
                message::MessageType::Hello(_) => {},
            };
        }
        thread::sleep(time::Duration::from_millis(100));
    }

    t.join();

}

fn handleCodecHeader(data: message::CodecHeaderData) -> Option<Box<Decoder>> {
    let decoder: Box<Decoder> = match data.codec.as_str() {
        "pcm" => Box::new(decoder::PCMDecoder),
        _ => Box::new(decoder::DummyDecoder)
    };
    decoder.setHeader(data);
    Some(decoder)
}
fn handleWireChunk(decoder: Option<Box<Decoder>>, data: message::WireChunkData, sink: &Sink) -> Option<Box<Decoder>> {
    if decoder.is_some() {
        let decoder = decoder.unwrap();
        let new_chunk = decoder.decode(data);
        /*let buffer = SamplesBuffer::new(1, 44100, new_chunk);*/
        sink.append(new_chunk);
        Some(decoder)
    } else {
        decoder
    }
}
fn handleServerSetting(data: message::ServerSettingsData) {

}
fn handleTime(data: message::TimeData) {

}
