use core::time::Duration;

use crate::libos::libos;

#[derive(Clone, Copy)]
pub struct SystemTime {
    nanos: u128,
}

pub const UNIX_EPOCH: SystemTime = SystemTime { nanos: 0 };

impl SystemTime {
    pub fn now() -> Self {
        let nanos = libos!(get_time()).unwrap();
        // println!("get_time -> {}", nanos);
        Self { nanos }
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Duration {
        let sub = self.nanos - earlier.nanos;
        Duration::new((sub / 1_000_000_000) as u64, (sub % 1_000_000_000) as u32)
    }

    pub fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }
}

pub fn sleep(dur: Duration) {
    let sec = dur.as_secs();
    let nsec = (dur.as_nanos() % 1_000_000_000) as u64;

    libos!(nanosleep(sec, nsec));
}
