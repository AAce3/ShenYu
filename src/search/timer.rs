use std::{cmp, time::Instant};

pub struct Timer {
    pub time_alloted: u64, //ms
    pub max_nodes: u64,
    pub maxdepth: u8,
    pub start_time: Instant,
    pub is_timed: bool,
    pub stopped: bool,
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
        }
    }
    pub fn check_time(&self) -> bool {
        self.start_time.elapsed().as_millis() as u64 >= self.time_alloted
    }
    pub fn allocate_time(timeleft: u64, inc: u64) -> u64 {
        let cannot_exceed = cmp::max(timeleft / 8, 1); // avoid allocating more than 1/8 of the time to avoid time pressure
        
        let timeleft = (timeleft / 30) + inc / 4;
        cmp::min(timeleft, cannot_exceed)
    }

    pub fn refresh(&mut self) {
        self.time_alloted = u64::MAX;
        self.max_nodes = u64::MAX;
        self.maxdepth = u8::MAX;
        self.start_time = Instant::now();
        self.is_timed = false;
        self.stopped = false;
    }
}
