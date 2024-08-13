use crate::{boards::get_board, sbi::set_timer};
use riscv::register::time;

const MSEC_PER_SEC: usize = 1000;

#[repr(C)]
pub struct TimeVal {
    pub sec: u64,  // 秒数
    pub usec: u64, // 微秒数
}

impl TimeVal {
    pub fn new(sec: u64, usec: u64) -> Self {
        TimeVal { sec, usec }
    }

    pub fn zero() -> Self {
        TimeVal { sec: 0, usec: 0 }
    }
}

pub fn get_timeval() -> TimeVal {
    let board = get_board();

    let tick = board.get_board_tick();
    let freq = board.board_clock_freq() as usize;

    let sec = tick / freq;
    let usec = (tick % freq) * 1_000_000 / freq;

    TimeVal {
        sec: sec as u64,
        usec: usec as u64,
    }
}

#[inline]
pub fn get_time() -> usize {
    time::read()
}

#[inline]
pub fn get_tick() -> usize {
    get_board().get_board_tick()
}

#[inline]
pub fn get_time_ms() -> usize {
    get_board().get_board_time_ms()
}

#[inline]
pub fn tick_to_ms(tick: usize) -> usize {
    let freq = get_board().board_clock_freq() as usize;
    tick * MSEC_PER_SEC / freq
}

#[inline]
pub fn ms_to_tick(ms: usize) -> usize {
    let freq = get_board().board_clock_freq() as usize;
    ms * freq / MSEC_PER_SEC
}

pub fn set_next_trigger() {
    // 10ms
    let time = get_time_ms() + 10;
    let tick = ms_to_tick(time);
    set_timer(tick);
}
