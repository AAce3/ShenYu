use std::{
    cmp,
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc,
    },
    time::Instant,
};

pub struct Timer {
    pub time_alloted: u64, //ms
    pub max_nodes: u64,
    pub maxdepth: u8,
    pub start_time: Instant,
    pub is_timed: bool,
    pub stopped: bool,
    pub recv: Option<Receiver<bool>>,
}
impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
impl Timer {
    pub fn new() -> Timer {
        Timer {
            time_alloted: u64::MAX,
            max_nodes: u64::MAX,
            maxdepth: u8::MAX,
            start_time: Instant::now(),
            is_timed: false,
            stopped: false,
            recv: None,
        }
    }
    pub fn check_time(&self) -> bool {
        let hasmsg = match self.recv.as_ref().unwrap().try_recv() {
            Ok(key) => key,
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        };
        self.start_time.elapsed().as_millis() as u64 >= self.time_alloted || hasmsg
    }
    pub fn allocate_time(timeleft: u64, inc: u64) -> u64 {
        let cannot_exceed = cmp::max(timeleft / 8, 1); // avoid allocating more than 1/8 of the time to avoid time pressure
        let timeleft = (timeleft / 30) + inc / 2;
        cmp::min(timeleft, cannot_exceed)
    }
}
