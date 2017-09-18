extern crate clap;
extern crate alsa;
extern crate byteorder;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::ffi::CString;
use clap::{Arg, App, SubCommand};
use alsa::{Direction, ValueOr};
use alsa::pcm::{PCM, HwParams, Format, Access, State};
use std::net::TcpStream;
use std::io::{Write, Read};

mod message;
use message::SnapMessageData;
fn main() {
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
            println!("{}: {} ({})", x.get_index(), x.get_name().unwrap(), x.get_longname().unwrap());
        }
    }

    //let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();


    println!("Connecting");
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

    let msg = message::Message::deserialize_from_socket(stream);
    println!("{:?}", msg);

}

