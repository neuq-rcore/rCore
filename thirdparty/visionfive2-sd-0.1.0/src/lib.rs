#![no_std]
#![allow(unused)]
extern crate alloc;

use crate::cmd::*;
use crate::register::*;
use crate::utils::*;
use bit_field::BitField;
use bit_struct::*;
use byte_slice_cast::AsByteSlice;
use core::fmt::{Display, Formatter};
use core::mem::size_of;
use log::{debug, error, info, trace, warn};
use preprint::pprintln;

mod cmd;
mod register;
mod utils;

enum DataTransType<'a> {
    None,
    Read(&'a mut [usize]),
    Write(&'a [usize]),
}

fn wait_ms_util_can_send_cmd(ms: usize) -> bool {
    wait_ms(ms, || {
        let mut cmd_reg = CmdReg::try_from(read_reg(CMD_REG)).unwrap();
        cmd_reg.start_cmd().get_raw() == 0
    });
    let mut cmd_reg = CmdReg::try_from(read_reg(CMD_REG)).unwrap();
    cmd_reg.start_cmd().get_raw() == 0
}

fn wait_ms_util_can_send_data(ms: usize) -> bool {
    wait_ms(ms, || {
        let mut status_reg = StatusReg::try_from(read_reg(STATUS_REG)).unwrap();
        status_reg.data_busy().get_raw() == 0
    });
    let mut status_reg = StatusReg::try_from(read_reg(STATUS_REG)).unwrap();
    status_reg.data_busy().get_raw() == 0
}

fn wait_ms_util_response(ms: usize) -> bool {
    wait_ms(ms, || {
        let mut raw_int_status_reg =
            RawInterruptStatusReg::try_from(read_reg(RAW_INT_STATUS_REG)).unwrap();
        let int = raw_int_status_reg.int_status().get();
        let mut raw_int_status = RawInterrupt::try_from(int).unwrap();
        raw_int_status.command_done().get_raw() == 1
    });
    let mut raw_int_status_reg =
        RawInterruptStatusReg::try_from(read_reg(RAW_INT_STATUS_REG)).unwrap();
    let int = raw_int_status_reg.int_status().get();
    let mut raw_int_status = RawInterrupt::try_from(int).unwrap();
    raw_int_status.command_done().get_raw() == 1
}

fn fifo_filled_cnt() -> usize {
    let mut status = StatusReg::try_from(read_reg(STATUS_REG)).unwrap();
    status.fifo_count().get_raw() as usize
}

fn send_cmd(
    cmd_type: Cmd,
    mut cmd: CmdReg,
    arg: CmdArg,
    data_trans_type: DataTransType,
) -> Option<[u32; 4]> {
    let res = wait_ms_util_can_send_cmd(0);
    assert!(res);
    if cmd.data_expected().get_raw() == 1 {
        let res = wait_ms_util_can_send_data(0);
        assert!(res)
    }
    info!("send cmd type:{:?}, value:{:#?}", cmd_type, cmd);
    // write arg
    write_reg(ARG_REG, arg.raw());
    write_reg(CMD_REG, cmd.raw());
    // Wait for cmd accepted
    let command_accept = wait_ms_util_can_send_cmd(0);
    info!("command accepted {}", command_accept);

    if cmd.response_expect().get_raw() == 1 {
        let res = wait_ms_util_response(0);
        debug!("wait_ms_util_response:{:?}", res);
    }

    if cmd.data_expected().get_raw() == 1 {
        let mut fifo_addr = FIFO_DATA_REG;
        match data_trans_type {
            DataTransType::Read(buffer) => {
                trace!("data_expected read....");
                let mut buf_offset = 0;
                wait_ms(0, || {
                    let mut raw_int_status_reg =
                        RawInterruptStatusReg::try_from(read_reg(RAW_INT_STATUS_REG)).unwrap();
                    let int = raw_int_status_reg.int_status().get();
                    let mut raw_int_status = RawInterrupt::try_from(int).unwrap();
                    if raw_int_status.rxdr().get_raw() == 1 {
                        debug!("RXDR....");
                        while fifo_filled_cnt() >= 2 {
                            buffer[buf_offset] = read_fifo::<usize>(fifo_addr);
                            buf_offset += 1;
                            fifo_addr += size_of::<usize>();
                        }
                    }
                    raw_int_status.dto().get_raw() == 1 || raw_int_status.have_error()
                });
                info!(
                    "buf_offset:{}, receive {} bytes",
                    buf_offset,
                    buf_offset * 8
                );
            }
            DataTransType::Write(buffer) => {
                let mut buf_offset = 0;
                wait_ms(0, || {
                    let raw_int_status = read_reg(RAW_INT_STATUS_REG);
                    let mut raw_int_status = RawInterrupt::try_from(raw_int_status as u16).unwrap();
                    if raw_int_status.txdr().get_raw() == 1 {
                        debug!("TXDR....");
                        // Hard coded FIFO depth
                        while fifo_filled_cnt() < 120 && buf_offset < buffer.len() {
                            write_fifo(fifo_addr, buffer[buf_offset]);
                            buf_offset += 1;
                            fifo_addr += size_of::<usize>();
                        }
                    }
                    raw_int_status.dto().get_raw() == 1 || raw_int_status.have_error()
                });
                info!("buf_offset:{}, send {} bytes", buf_offset, buf_offset * 8);
            }
            _ => {
                panic!("Not implemented")
            }
        }
        debug!("Current FIFO count: {}", fifo_filled_cnt());
    }
    // Clear interrupt by writing 1
    let raw_int_status = read_reg(RAW_INT_STATUS_REG);
    write_reg(RAW_INT_STATUS_REG, raw_int_status);
    // check error
    let mut raw_int_status = RawInterruptStatusReg::try_from(raw_int_status).unwrap();
    let mut raw_int_status = RawInterrupt::try_from(raw_int_status.int_status().get()).unwrap();
    let resp = [
        read_reg(RESP0_REG),
        read_reg(RESP1_REG),
        read_reg(RESP2_REG),
        read_reg(RESP3_REG),
    ];
    if raw_int_status.have_error() {
        warn!("card has error {:#?}", raw_int_status);
        warn!("cmd {:#?}", cmd);
        warn!("resp {:x?}", resp[0]);
        return None;
    }
    Some(resp)
}

fn reset_clock() {
    // disable clock
    let mut clock_enable = ClockEnableReg::new(0, 0);
    // write to CLOCK_ENABLE_REG
    write_reg(CLOCK_ENABLE_REG, clock_enable.raw());
    // send reset clock command
    let mut clock_cmd = CmdReg::try_from(0).unwrap();
    clock_cmd.start_cmd().set(u1!(1));
    clock_cmd.wait_prvdata_complete().set(u1!(1));
    clock_cmd.update_clock_registers_only().set(u1!(1));
    send_cmd(
        Cmd::ResetClock,
        clock_cmd,
        CmdArg::new(0),
        DataTransType::None,
    );
    // set clock divider to 400kHz (low)
    let clock_divider = ClockDividerReg::new(0, 0, 0, 4);
    write_reg(CLK_DIVIDER_REG, clock_divider.raw());
    // send_cmd(Cmd::ResetClock,clock_disable_cmd,CmdArg::new(0));
    // enable clock
    clock_enable.clk_enable().set(1);
    write_reg(CLOCK_ENABLE_REG, clock_enable.raw());
    // send reset clock command
    send_cmd(
        Cmd::ResetClock,
        clock_cmd,
        CmdArg::new(0),
        DataTransType::None,
    );
    info!(
        "now clk enable {:#?}",
        ClockEnableReg::try_from(read_reg(CLOCK_ENABLE_REG)).unwrap()
    );
    pprintln!("reset clock success");
}

fn reset_fifo() {
    let mut ctrl = ControlReg::try_from(read_reg(CTRL_REG)).unwrap();
    ctrl.fifo_reset().set(u1!(1));
    // todo!(why write to fifo data)?
    // write_reg(CTRL_REG,ctrl.raw());
    write_reg(FIFO_DATA_REG, ctrl.raw());
    pprintln!("reset fifo success");
}

fn reset_dma() {
    let mut buf_mode_reg = BusModeReg::try_from(read_reg(BUS_MODE_REG)).unwrap();
    buf_mode_reg.de().set(u1!(0));
    buf_mode_reg.swr().set(u1!(1));
    write_reg(BUS_MODE_REG, buf_mode_reg.raw());
    let mut ctrl = ControlReg::try_from(read_reg(CTRL_REG)).unwrap();
    // ctrl.dma_enable().set(u1!(0));
    ctrl.dma_reset().set(u1!(1));
    ctrl.use_internal_dmac().set(u1!(0));
    write_reg(CTRL_REG, ctrl.raw());
    pprintln!("reset dma success");
}

fn set_transaction_size(blk_size: u32, byte_count: u32) {
    let val = blk_size as u16;
    let mut blk_size = BlkSizeReg::try_from(0).unwrap();
    blk_size.block_size().set(val);
    write_reg(BLK_SIZE_REG, blk_size.raw());
    let value = byte_count;
    let mut byte_count = ByteCountReg::try_from(0).unwrap();
    byte_count.byte_count().set(value);
    write_reg(BYTE_CNT_REG, byte_count.raw());
}

fn test_read() {
    pprintln!("test read, try read 0 block");
    set_transaction_size(512, 512);
    let cmd17 = CmdReg::from(Cmd::ReadSingleBlock);
    let arg = CmdArg::new(0);
    let mut buffer: [usize; 64] = [0; 64];
    let _resp = send_cmd(
        Cmd::ReadSingleBlock,
        cmd17,
        arg,
        DataTransType::Read(&mut buffer),
    )
    .unwrap();
    info!("Current FIFO count: {}", fifo_filled_cnt());
    let byte_slice = buffer.as_byte_slice();
    pprintln!("sd header 16bytes: {:x?}", &byte_slice[..16]);
}

/// for test driver
#[allow(unused)]
fn test_write_read() {
    set_transaction_size(512, 512);
    // write a block data
    let cmd24 = CmdReg::from(Cmd::WriteSingleBlock);
    let arg = CmdArg::new(0);
    let mut buffer: [usize; 64] = [0; 64];
    buffer.fill(usize::MAX);
    let _resp = send_cmd(
        Cmd::WriteSingleBlock,
        cmd24,
        arg,
        DataTransType::Write(&buffer),
    )
    .unwrap();
    // info!("resp csr: {:#?}",resp[0]); //csr reg
    info!("Current FIFO count: {}", fifo_filled_cnt());
    // read a block data
    let cmd17 = CmdReg::from(Cmd::ReadSingleBlock);
    let arg = CmdArg::new(0);
    let mut buffer: [usize; 64] = [0; 64];
    let _resp = send_cmd(
        Cmd::ReadSingleBlock,
        cmd17,
        arg,
        DataTransType::Read(&mut buffer),
    )
    .unwrap();
    // info!("resp csr: {:#?}",resp[0]); //csr reg
    info!("Current FIFO count: {}", fifo_filled_cnt());
    let byte_slice = buffer.as_byte_slice();
    debug!("Head 16 bytes: {:#x?}", &byte_slice[..16]);
}

// send acmd51 to read csr reg
fn check_bus_width(rca: u32) -> usize {
    let cmd55 = CmdReg::from(Cmd::AppCmd);
    let cmd_arg = CmdArg::new(rca << 16);
    let _resp = send_cmd(Cmd::AppCmd, cmd55, cmd_arg, DataTransType::None).unwrap();
    // send acmd51
    // 1. set transact size
    set_transaction_size(8, 8);
    // 2. send command
    let acmd51 = CmdReg::from(Cmd::SendScr);
    let mut buffer: [usize; 64] = [0; 64]; // 512B
    send_cmd(
        Cmd::SendScr,
        acmd51,
        CmdArg::new(0),
        DataTransType::Read(&mut buffer),
    );
    info!("Current FIFO count: {}", fifo_filled_cnt()); //2
    let resp = u64::from_be(read_fifo::<u64>(FIFO_DATA_REG));
    pprintln!("Bus width supported: {:b}", (resp >> 48) & 0xF);
    info!("Current FIFO count: {}", fifo_filled_cnt()); //0
    0
}

fn check_csd(rca: u32) {
    let cmd = CmdReg::from(Cmd::SendCsd);
    let resp = send_cmd(
        Cmd::SendCsd,
        cmd,
        CmdArg::new(rca << 16),
        DataTransType::None,
    )
    .unwrap();
    let status = resp[0];
    pprintln!("status: {:b}", status);
}

fn select_card(rca: u32) {
    let cmd7 = CmdReg::from(Cmd::SelectCard);
    let cmd_arg = CmdArg::new(rca << 16);
    let resp = send_cmd(Cmd::SelectCard, cmd7, cmd_arg, DataTransType::None).unwrap();
    let r1 = resp[0];
    info!("status: {:b}", r1);
}

fn check_rca() -> u32 {
    let cmd3 = CmdReg::from(Cmd::SendRelativeAddr);
    let resp = send_cmd(
        Cmd::SendRelativeAddr,
        cmd3,
        CmdArg::new(0),
        DataTransType::None,
    )
    .unwrap();
    let rca = resp[0] >> 16;
    info!("rca: {:#x}", rca);
    info!("card status: {:b}", resp[0] & 0xffff);
    rca
}

fn check_cid() {
    let cmd2 = CmdReg::from(Cmd::AllSendCid);
    let resp = send_cmd(Cmd::AllSendCid, cmd2, CmdArg::new(0), DataTransType::None);
    if let Some(resp) = resp {
        // to 128 bit
        let resp = unsafe { core::mem::transmute::<[u32; 4], u128>(resp) };
        let cid = Cid::new(resp);
        pprintln!("cid: {}", cid.fmt());
    }
}

fn check_version() -> u8 {
    // check voltage
    let cmd8 = CmdReg::from(Cmd::SendIfCond);
    let cmd8_arg = CmdArg::new(0x1aa);
    let resp = send_cmd(Cmd::SendIfCond, cmd8, cmd8_arg, DataTransType::None).unwrap();
    if (resp[0] & 0xaa) == 0 {
        error!("card {} unusable", 0);
        pprintln!("card version: 1.0");
        return 1;
    }
    pprintln!("card voltage: {:#x?}", resp[0]);
    pprintln!("card version: 2.0");
    2
}

fn check_big_support(sleep: fn(usize)) -> bool {
    loop {
        // send cmd55
        let cmd55 = CmdReg::from(Cmd::AppCmd);
        send_cmd(Cmd::AppCmd, cmd55, CmdArg::new(0), DataTransType::None);
        let cmd41 = CmdReg::from(Cmd::SdSendOpCond);
        let cmd41_arg = CmdArg::new((1 << 30) | (1 << 24) | 0xFF8000);
        let resp = send_cmd(Cmd::SdSendOpCond, cmd41, cmd41_arg, DataTransType::None).unwrap();
        info!("ocr: {:#x?}", resp[0]);
        let ocr = resp[0];
        if ocr.get_bit(31) {
            pprintln!("card is ready");
            if ocr.get_bit(30) {
                pprintln!("card is high capacity");
            } else {
                pprintln!("card is standard capacity");
            }
            break;
        }
        sleep(100);
    }
    true
}

fn init_sdcard(sleep: fn(usize)) {
    // read DETECT_REG
    let detect = read_reg(CDETECT_REG);
    info!("detect: {:#?}", CDetectReg::try_from(detect).unwrap());
    // read POWER_REG
    let power = read_reg(POWER_REG);
    info!("power: {:#?}", PowerReg::try_from(power).unwrap());
    // read CLOCK_ENABLE_REG
    let clock_enable = read_reg(CLOCK_ENABLE_REG);
    info!(
        "clock_enable: {:#?}",
        ClockEnableReg::try_from(clock_enable).unwrap()
    );
    // read CARD_TYPE_REG
    let card_type = read_reg(CTYPE_REG);
    info!(
        "card_type: {:#?}",
        CardTypeReg::try_from(card_type).unwrap()
    );
    // read Control Register
    let control = read_reg(CTRL_REG);
    info!("control: {:#?}", ControlReg::try_from(control).unwrap());
    // read  bus mode register
    let bus_mode = read_reg(BUS_MODE_REG);
    info!(
        "bus_mode(DMA): {:#?}",
        BusModeReg::try_from(bus_mode).unwrap()
    );
    // read DMA Descriptor List Base Address Register
    let dma_desc_base_lower = read_reg(DBADDRL_REG);
    let dma_desc_base_upper = read_reg(DBADDRU_REG);
    let dma_desc_base: usize = dma_desc_base_lower as usize | (dma_desc_base_upper as usize) << 32;
    info!("dma_desc_base: {:#x?}", dma_desc_base);
    // read clock divider register
    let clock_divider = read_reg(CLK_DIVIDER_REG);
    info!(
        "clock_divider: {:#?}",
        ClockDividerReg::try_from(clock_divider).unwrap()
    );

    // reset card clock to 400Mhz
    reset_clock();
    // reset fifo
    reset_fifo();

    // set data width --> 1bit
    let mut ctype = CardTypeReg::try_from(0).unwrap();
    ctype.card_width4_1().set(0);
    write_reg(CTYPE_REG, ctype.raw());

    // reset dma
    reset_dma();

    let ctrl = ControlReg::try_from(read_reg(CTRL_REG)).unwrap();
    info!("ctrl: {:#?}", ctrl);

    // go idle state
    let cmd0 = CmdReg::from(Cmd::GoIdleState);
    // cmd0.response_expect().set(u1!(0));
    send_cmd(Cmd::GoIdleState, cmd0, CmdArg::new(0), DataTransType::None);
    pprintln!("card is in idle state");

    check_version();

    check_big_support(sleep);

    check_cid();
    let rca = check_rca();
    pprintln!("rca: {:#x?}", rca);
    check_csd(rca);

    // let raw_int_status = RawInterruptStatusReg::try_from(read_reg(RAW_INT_STATUS_REG)).unwrap();
    // pprintln!("RAW_INT_STATUS_REG: {:#?}", raw_int_status);

    select_card(rca);

    let mut status = StatusReg::try_from(read_reg(STATUS_REG)).unwrap();
    info!("Now FIFO Count is {}", status.fifo_count().get_raw());

    // check bus width
    check_bus_width(rca);
    // try read a block data
    test_read();
    // test_write_read();

    info!(
        "CTRL_REG: {:#?}",
        ControlReg::try_from(read_reg(CTRL_REG)).unwrap()
    );
    let raw_int_status = RawInterruptStatusReg::try_from(read_reg(RAW_INT_STATUS_REG)).unwrap();
    info!("RAW_INT_STATUS_REG: {:#?}", raw_int_status);
    // Clear interrupt by writing 1
    write_reg(RAW_INT_STATUS_REG, raw_int_status.raw());

    pprintln!("init sd success");
}

#[derive(Debug, Copy, Clone)]
pub enum Vf2SdDriverError {
    InitError,
    ReadError,
    WriteError,
    TimeoutError,
    UnknownError,
}

impl Display for Vf2SdDriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Vf2SdDriverError::InitError => write!(f, "init error"),
            Vf2SdDriverError::ReadError => write!(f, "read error"),
            Vf2SdDriverError::WriteError => write!(f, "write error"),
            Vf2SdDriverError::TimeoutError => write!(f, "timeout error"),
            Vf2SdDriverError::UnknownError => write!(f, "unknown error"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Vf2SdDriverError>;

fn read_block(block: usize, buf: &mut [u8]) -> Result<usize> {
    let mut buf = unsafe {
        let ptr = buf.as_mut_ptr() as *mut usize;
        core::slice::from_raw_parts_mut(ptr, 64)
    };
    assert_eq!(buf.len(), 64);
    set_transaction_size(512, 512);
    let cmd17 = CmdReg::from(Cmd::ReadSingleBlock);
    let arg = CmdArg::new(block as u32);
    let _resp = send_cmd(
        Cmd::ReadSingleBlock,
        cmd17,
        arg,
        DataTransType::Read(&mut buf),
    )
    .unwrap();
    info!("Current FIFO count: {}", fifo_filled_cnt());
    Ok(buf.len())
}

fn write_block(block: usize, buf: &[u8]) -> Result<usize> {
    let buf = unsafe {
        let ptr = buf.as_ptr() as *mut usize;
        core::slice::from_raw_parts(ptr, 64)
    };
    assert_eq!(buf.len(), 64);
    set_transaction_size(512, 512);
    let cmd24 = CmdReg::from(Cmd::WriteSingleBlock);
    let arg = CmdArg::new(block as u32);
    let _resp = send_cmd(Cmd::WriteSingleBlock, cmd24, arg, DataTransType::Write(buf)).unwrap();
    info!("Current FIFO count: {}", fifo_filled_cnt());
    Ok(buf.len())
}

pub struct Vf2SdDriver {
    sleep: fn(usize),
}

impl Vf2SdDriver {
    pub fn new(sleep: fn(usize)) -> Self {
        Self { sleep }
    }
    pub fn init(&self) {
        init_sdcard(self.sleep);
    }
    pub fn read_block(&self, block: usize, buf: &mut [u8]) {
        assert_eq!(buf.len(), 512);
        read_block(block, buf).unwrap();
    }
    pub fn write_block(&self, block: usize, buf: &[u8]) {
        assert_eq!(buf.len(), 512);
        write_block(block, buf).unwrap();
    }
}
