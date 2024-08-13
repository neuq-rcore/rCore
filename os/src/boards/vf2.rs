// Reference:
// https://github.com/LiuJiLan/RVOS_On_VisionFive2
// https://forum.rvspace.org/t/demo-rvos-visionfive2/2979

use super::IBoard;

#[derive(Clone, Copy)]
pub struct VF2Board;

impl VF2Board {
    pub fn new() -> Self {
        VF2Board
    }
}

impl IBoard for VF2Board {
    fn board_name(&self) -> &'static str {
        "StarFive VisionFive 2"
    }

    fn board_clock_freq(&self) -> u64 {
        4_000_000
    }

    fn mmio(&self) -> &[(usize, usize)] {
        &[
            (0x1601_0000, 0x1_0000 * 2), // Currently only cover SD card
                                         // TODO
        ]
    }

    fn memory_end(&self) -> usize {
        0x1_8000_0000
    }

    fn bus0(&self) -> usize {
        // bus1 is SD card
        0x1601_0000
    }

    fn bus_width(&self) -> usize {
        0x1_0000
    }
}
