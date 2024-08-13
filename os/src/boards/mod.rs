mod vf2;
mod virt;

use core::arch::asm;

use alloc::{boxed::Box, string::String, sync::Arc};
use log::info;
use riscv::register::mimpid;

use crate::timer;

static mut BOARD: Option<Arc<dyn IBoard>> = None;

pub trait IBoard {
    // Board metadata
    fn board_name(&self) -> &'static str;
    fn board_clock_freq(&self) -> u64;
    fn mmio(&self) -> &[(usize, usize)];
    fn memory_end(&self) -> usize;

    fn get_board_tick(&self) -> usize {
        timer::get_time()
    }

    fn sleep(&self, ms: usize) {
        let start = self.get_board_tick();
        let end = start + ms * self.board_clock_freq() as usize / 1000;

        while self.get_board_tick() < end {
            unsafe {
                asm!("nop");
            }
        }
    }

    fn bus0(&self) -> usize;
    fn bus_width(&self) -> usize;

    fn mmc_driver(&self, device_id: usize) -> usize {
        self.bus0() + device_id * self.bus_width()
    }

    fn get_board_time_ms(&self) -> usize {
        (self.get_board_tick() * 1000 / self.board_clock_freq() as usize)
    }
}

fn init_board() -> Arc<dyn IBoard> {
    #[cfg(feature = "board_virt")]
    {
        Arc::new(virt::VirtBoard::new())
    }

    #[cfg(feature = "board_vf2")]
    {
        Arc::new(vf2::VF2Board::new())
    }

    #[cfg(not(any(feature = "board_virt", feature = "board_vf2")))]
    {
        panic!("No board feature enabled, please enable one of them with \'--features=board_virt\' or \'--features=board_vf2\'");
    }
}

fn human_friendly_hz(mut hz: u64) -> String {
    let mut unit = 0;

    // Don't think we should consider CPUs that has a frequency higher than 1000 GHz
    let units = ["Hz", "KHz", "MHz", "GHz"];

    // Multiply by 100 to avoid losing precision
    while hz >= 100 * 1000 && unit < units.len() - 1 {
        hz = (hz + 5) / 10; // Round to nearest
        unit += 1;
    }

    let integer = hz / 100;
    let fractional = hz % 100;

    format!("{}.{:02} {}", integer, fractional, units[unit])
}

pub fn debug_board_info(board: Arc<dyn IBoard>) {
    info!("Board: {}", board.board_name());

    // Used to identify the machine implementation
    // let mimpid = mimpid::read().map(|id| id.bits()).unwrap_or(0);
    // info!("Mimpid: {:#x}", mimpid);

    let freq = board.board_clock_freq();
    let freq = human_friendly_hz(freq);
    info!("Clock: {}", freq);

    info!("Memory end: {:#016x}", board.memory_end());

    info!("MMIO:");
    for &(base, size) in board.mmio() {
        info!("  {:#016x} - {:#016x}", base, base + size);
    }
}

pub fn get_board() -> Arc<dyn IBoard> {
    match unsafe { BOARD.as_ref() } {
        None => unsafe {
            let board = init_board();
            BOARD = Some(board.clone());

            board
        },
        Some(board) => board.clone(),
    }
}
