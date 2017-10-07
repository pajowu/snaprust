use message::TimeVal;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub struct TimeProvider {
    times: Vec<usize>
}

impl TimeProvider {
    pub fn new() -> Self {
        TimeProvider {
            times: Vec::new()
        }
    }
    pub fn add_time(&mut self, time: TimeVal) {
        let diff = (time.sec as usize)*1000 + (time.usec as usize / 1000);
        self.times.push(diff);
        if self.times.len() > 200 {
            let l = self.times.len() - 200;
            self.times = self.times.split_off(l);
        }
    }
    pub fn get_diff_to_server(&self) -> usize {
        let l = self.times.len();
        let i = self.times.iter();
        let s: usize = i.sum();
        if l == 0 {
            0
        } else {
            s / l
        }

    }

    pub fn get_server_time(&self) -> usize {
        let time = TimeVal::new();
        let current_time = ((time.sec as usize)*1000 + (time.usec as usize / 1000)) as usize;
        let diff = self.get_diff_to_server() as usize;
        current_time + diff
    }
}
