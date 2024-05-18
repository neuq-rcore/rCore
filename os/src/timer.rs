use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

const MSEC_PER_SEC: usize = 1000;

#[repr(C)]
pub struct TimeVal{
    pub sec: u64,  // 自 Unix 纪元起的秒数
    pub usec: u64, // 微秒数
}

pub fn get_timeval() -> TimeVal {
    let now = get_time();

    let sec = (time_to_ms(now) / 1000) as u64;
    let usec = (now * 2 / 25) as u64;

    TimeVal {
        sec,
        usec,
    }
}

#[inline]
pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_ms() -> usize {
    time_to_ms(get_time())
}

#[inline]
pub fn time_to_ms(time: usize) -> usize {
    time / (CLOCK_FREQ / MSEC_PER_SEC)
}

pub fn set_next_trigger() {
    // 10ms
    set_timer(get_time() + CLOCK_FREQ / 100);
}
