use std::collections::VecDeque;
use std::sync::Mutex;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH, Instant};

use message;

use log;
use simplelog;

pub struct ClientConnection {
    stream: TcpStream,
    send_message_channel: mpsc::Receiver<message::Message>,
    recv_message_channel: mpsc::Sender<message::Message>,
    last_timesync: Instant
}

impl ClientConnection {
    pub fn start(host: &str)
        -> (ClientConnection, mpsc::Sender<message::Message>, mpsc::Receiver<message::Message>) {
        let mut stream = TcpStream::connect(host).unwrap();
        stream.set_nonblocking(true);
        let (s_msg_tx, s_msg_rx) = mpsc::channel();
        let (r_msg_tx, r_msg_rx) = mpsc::channel();
        (ClientConnection {
            stream: stream,
            send_message_channel: s_msg_rx,
            recv_message_channel: r_msg_tx,
            last_timesync: Instant::now(),
        }, s_msg_tx, r_msg_rx)
    }

    pub fn worker(&mut self) {
        let sleep_period = time::Duration::from_millis(100);
        let time_sync_every = 500;

        loop {

            let time_since_last_sync = self.last_timesync.elapsed();
            let diff = time_since_last_sync.as_secs()*1000 + (time_since_last_sync.subsec_nanos() / 1000000) as u64;
            if diff > time_sync_every {
                self.last_timesync = Instant::now();
                let data = message::TimeData {
                    latency: message::TimeVal { sec: 0, usec: 0 }
                };

                let time_msg = message::Message {
                    type_: message::MessageType::Time(data),
                    id: 0,
                    refers_to: 0,
                    recieved: message::TimeVal::new(),
                    sent: message::TimeVal::new()
                };
                let msg = time_msg.serialize();
                let msg = msg.as_slice();
                self.stream.write_all(&msg[..]);
                info!("Timesync!");
            }

            while let Ok(msg) = self.send_message_channel.try_recv() {
                debug!("Send: {:?}", msg);
                let msg = msg.serialize();
                let msg = msg.as_slice();
                self.stream.write_all(&msg[..]);
            }

            while let Ok(msg) = message::Message::deserialize_from_socket(&self.stream) {
                debug!("Read message: {:?}", msg);
                self.recv_message_channel.send(msg);
            }

            thread::sleep(sleep_period);
        }
    }
}

pub fn fill_queue(sender: mpsc::Sender<message::Message>, stream: TcpStream) {
    loop {
        let msg = message::Message::deserialize_from_socket(&stream);

        sender.send(msg.unwrap()).unwrap();
    }

}
