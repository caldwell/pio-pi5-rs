// Copyright © 2025 David Caldwell <david@porkrind.org>
// SPDX-License-Identifier: BSD-3-Clause

use std::{ffi::c_void, fs::File, os::fd::AsRawFd, path::{Path, PathBuf}};

use libc::c_ulong;

use crate::{proc_pio::*, Chip, Error, PIOInstance, SmConfig, GPIOS_MASK, GPIO_COUNT, GPIO_FUNC_PIO, INSTRUCTION_COUNT};
use crate::gpio::*;
use crate::ioctl::*;

pub struct Rp1PIO {
    base: PIOInstance,
    devname: PathBuf,
    fd: std::os::fd::OwnedFd,
}

impl Rp1PIO {
    pub fn new(index: usize) -> Result<Rp1PIO, Error> {
        let devname = format!("/dev/pio{index}").into();
        Ok(Rp1PIO {
            base: PIOInstance::reserve(index)?,
            fd: File::open(&devname)?.into(),
            devname,
        })
    }

    pub fn chip(&self) -> &Chip {
        &self.base.chip
    }

    pub fn devname(&self) -> &Path {
        self.devname.as_path()
    }

    unsafe fn rp1_ioctl_mut_ptr(&self, request: c_ulong, args: *mut c_void) -> Result<u32, Error> {
        const NEG_EREMOTEIO: i32 = -libc::EREMOTEIO;
        const NEG_ETIMEDOUT: i32 = -libc::ETIMEDOUT;
        match unsafe {
            libc::ioctl(self.fd.as_raw_fd(), request, args)
        } {
            NEG_EREMOTEIO   => Err(Error::RemoteIOErr),
            NEG_ETIMEDOUT   => Err(Error::TimedOut),
            -1              => Err(std::io::Error::last_os_error())?,
            r@ ..-1         => Err(Error::Unknown(r)),
            r@ 0..          => Ok(r as u32),
        }
    }
    unsafe fn rp1_ioctl_const_ptr(&self, request: c_ulong, args: *const c_void) -> Result<u32, Error> {
        unsafe { self.rp1_ioctl_mut_ptr(request, args as *mut c_void) }
    }

    fn rp1_ioctl<A>(&self, request: c_ulong, args: &A) -> Result<u32, Error> {
        unsafe { self.rp1_ioctl_const_ptr(request, args as *const A as *const c_void) }
    }
    fn rp1_ioctl_mut<A>(&self, request: c_ulong, args: &mut A) -> Result<u32, Error> {
        unsafe { self.rp1_ioctl_mut_ptr(request, args as *mut A as *mut c_void) }
    }

    fn check_sm_param(&self, sm: u16) -> Result<(), Error> {
        if sm < self.base.chip.sm_count {
            Ok(())
        } else {
            Err(Error::BadSM { sm, max:self.base.chip.sm_count })
        }
    }

    fn check_sm_mask(&self, mask: u16) -> Result<(), Error> {
        if mask < (1 << self.base.chip.sm_count) {
            Ok(())
        } else {
            Err(Error::BadSMMask { sm_mask: mask, max: (1 << self.base.chip.sm_count) - 1 })
        }
    }

    pub fn sm_config_xfer(&self, sm: u16, dir: XferDir, buf_size: u32, buf_count: u32) -> Result<(), Error> {
        self.check_sm_param(sm)?;
        if buf_size > 0xffff || buf_count > 0xffff {
            let args = SmConfigXfer32Args { sm, dir: dir as u16, buf_size, buf_count };
            self.rp1_ioctl(PIO_IOC_SM_CONFIG_XFER32, &args)
        } else {
            let args = SmConfigXferArgs { sm, dir: dir as u16, buf_size: buf_size as u16, buf_count: buf_count as u16 };
            self.rp1_ioctl(PIO_IOC_SM_CONFIG_XFER, &args)
        }
            .map(|_| ())
    }

    pub fn sm_xfer_data<T>(&self, sm: u16, dir: u16, data_bytes: u32, data: &T) -> Result<(), Error> {
        self.check_sm_param(sm)?;
        if data_bytes > 0xffff {
            let args = SmXferData32Args { sm, dir, data_bytes, data: data as *const T as *const c_void };
            self.rp1_ioctl(PIO_IOC_SM_XFER_DATA32, &args)
        } else {
            let args = SmXferDataArgs { sm, dir, rsvd: 0, data_bytes: data_bytes as u16, data: data as *const T as *const c_void };
            self.rp1_ioctl(PIO_IOC_SM_XFER_DATA, &args)
        }
            .map(|_| ())
    }

    fn add_program_args(&self, program: &PioProgram, offset: Option<u16>) -> Result<AddProgramArgs, Error> {
        let offset = match (program.origin, offset) {
            (..0,         None)         => !0,
            (..0,         Some(offset)) => offset as u16,
            (origin, None)              => origin as u16,
            (origin, Some(offset)) if origin == offset as i8
                                        => origin as u16,
            (origin, Some(offset))      =>
                Err(Error::OffsetOriginMismatch { origin: origin as u8, offset })?,
        };
        if offset != !0 && offset >= INSTRUCTION_COUNT {
            Err(Error::OffsetTooLarge { offset, max: INSTRUCTION_COUNT })?;
        }
        if program.instructions.len() >= INSTRUCTION_COUNT as usize {
            Err(Error::TooManyInstructions { instructions: program.instructions.len(), max: INSTRUCTION_COUNT })?;
        }
        if offset != !0 && offset as usize + program.instructions.len() > INSTRUCTION_COUNT as usize {
            Err(Error::TooManyInstructions { instructions: program.instructions.len(), max: INSTRUCTION_COUNT - offset })?;
        }
        let mut args = AddProgramArgs {
            num_instrs: program.instructions.len() as u16,
            origin: offset,
            instrs: [0; INSTRUCTION_COUNT as usize],
        };
        for (i, insn) in program.instructions.iter().enumerate() {
            args.instrs[i] = *insn;
        }
        Ok(args)
    }

    pub fn can_add_program_at_offset(&self, program: &PioProgram, offset: Option<u16>) -> Result<bool, Error> {
        let args = self.add_program_args(program, offset)?;
        self.rp1_ioctl(PIO_IOC_CAN_ADD_PROGRAM, &args)
            .map(|r| r > 0)
    }

    pub fn can_add_program(&self, program: &PioProgram) -> Result<bool, Error> {
        self.can_add_program_at_offset(program, None)
    }

    pub fn add_program_at_offset(&self, program: &PioProgram, offset: Option<u16>) -> Result<u16, Error> {
        let args = self.add_program_args(program, offset)?;
        self.rp1_ioctl(PIO_IOC_ADD_PROGRAM, &args)
            .map(|offset| offset as u16)
    }

    pub fn add_program(&self, program: &PioProgram) -> Result<u16, Error> {
        self.add_program_at_offset(program, None)
    }

    pub fn remove_program(&self, program: &PioProgram, offset: Option<u16>) -> Result<bool, Error> {
        let args = RemoveProgramArgs { num_instrs: program.instructions.len() as u16,
                                           origin: offset.unwrap_or(!0),
        };
        if program.instructions.len() >= INSTRUCTION_COUNT as usize {
            Err(Error::TooManyInstructions { instructions: program.instructions.len(), max: INSTRUCTION_COUNT })?;
        }
        if args.origin != !0 && args.origin as usize + program.instructions.len() > INSTRUCTION_COUNT as usize {
            Err(Error::TooManyInstructions { instructions: program.instructions.len(), max: INSTRUCTION_COUNT - args.origin })?;
        }
        self.rp1_ioctl(PIO_IOC_REMOVE_PROGRAM, &args)
            .map(|r| r != 0)
    }

    pub fn clear_instruction_memory(&self) -> Result<bool, Error> {
        unsafe {
            self.rp1_ioctl_const_ptr(PIO_IOC_CLEAR_INSTR_MEM, std::ptr::null::<c_void>() as *const c_void)
        }
        .map(|r| r != 0)
    }

    pub fn sm_claim(&self, sm: u16) -> Result<StateMachine<'_>, Error> {
        self.check_sm_param(sm)?;
        let args = SmClaimArgs { mask: 1 << sm };
        self.rp1_ioctl(PIO_IOC_SM_CLAIM, &args)?;
        Ok(StateMachine { pio: &self, index: sm })
    }

    pub fn sm_claim_mask(&self, mask: u16) -> Result<Vec<StateMachine<'_>>, Error> {
        self.check_sm_mask(mask)?;
        let args = SmClaimArgs { mask };
        self.rp1_ioctl(PIO_IOC_SM_CLAIM, &args)?;
        (0..4).filter_map(|sm| match 1<<sm {
            0 => None,
            _ => Some(Ok(StateMachine { pio: &self, index: sm })),
        }).collect()
    }

    pub fn sm_claim_unused(&self) -> Result<StateMachine<'_>, Error> {
        let args = SmClaimArgs { mask: 0 };
        Ok(StateMachine { pio: &self,
                          index: self.rp1_ioctl(PIO_IOC_SM_CLAIM, &args)? as u16 })
    }

    pub fn sm_set_enabled_mask(&self, mask: u16, enabled:bool) -> Result<(), Error> {
        self.check_sm_mask(mask)?;
        let args = SmSetEnabledArgs { mask, enable: enabled.into(), rsvd:0 };
        self.rp1_ioctl(PIO_IOC_SM_SET_ENABLED, &args)
            .map(|_| ())
    }

    pub fn sm_restart_mask(&self, mask: u16) -> Result<(), Error> {
        self.check_sm_mask(mask)?;
        let args = SmRestartArgs { mask };
        self.rp1_ioctl(PIO_IOC_SM_RESTART, &args)
            .map(|_| ())
    }

    pub fn sm_clkdiv_restart_mask(&self, mask: u16) -> Result<(), Error> {
        self.check_sm_mask(mask)?;
        let args = SmClkdivRestartArgs { mask };
        self.rp1_ioctl(PIO_IOC_SM_CLKDIV_RESTART, &args)
            .map(|_| ())
    }

    pub fn sm_enable_sync(&self, mask: u16) -> Result<(), Error> {
        self.check_sm_mask(mask)?;
        let args = SmEnableSyncArgs { mask };
        self.rp1_ioctl(PIO_IOC_SM_ENABLE_SYNC, &args)
            .map(|_| ())
    }


    ///// GPIO Stuff. FIXME: Should this go somehwere else?? Or perhaps be folded into rpi-pal?

    fn check_gpio(&self, gpio: u16) -> Result<(), Error> {
        if gpio < GPIO_COUNT as u16 { Ok(()) }
        else { Err(Error::BadGPIO { gpio, max: GPIO_COUNT }) }
    }

    pub fn gpio_init(&self, gpio: u16) -> Result<(), Error> { // static void rp1_gpio_init(PIO pio, uint gpio)
        self.check_gpio(gpio)?;
        let args = GpioInitArgs { gpio };
        self.rp1_ioctl(PIO_IOC_GPIO_INIT, &args)
            .map(|_| ())
    }

    pub fn gpio_set_function(&self, gpio: u16, func: Function) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetFunctionArgs { gpio, func: func as u16 };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_FUNCTION, &args)
            .map(|_| ())
    }

    pub fn set_pulls(&self, gpio: u16, up: bool, down: bool) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetPullsArgs { gpio, up: up.into(), down: down.into() };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_PULLS, &args)
            .map(|_| ())
    }

    pub fn gpio_set_outover(&self, gpio: u16, value: u16) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetArgs { gpio, value };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_OUTOVER, &args)
            .map(|_| ())
    }

    pub fn gpio_set_inover(&self, gpio: u16, value: u16) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetArgs { gpio, value };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_INOVER, &args)
            .map(|_| ())
    }

    pub fn gpio_set_oeover(&self, gpio: u16, value: u16) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetArgs { gpio, value };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_OEOVER, &args)
            .map(|_| ())
    }

    pub fn gpio_set_input_enabled(&self, gpio: u16, enabled: bool) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetArgs { gpio, value: enabled.into() };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_INPUT_ENABLED, &args)
            .map(|_| ())
    }

    pub fn gpio_set_drive_strength(&self, gpio: u16, drive: DriveStrength) -> Result<(), Error> {
        self.check_gpio(gpio)?;
        let args = GpioSetArgs { gpio, value: drive as u16 };
        self.rp1_ioctl(PIO_IOC_GPIO_SET_DRIVE_STRENGTH, &args)
            .map(|_| ())
    }

    pub fn pio_gpio_init(&self, pin: u16) -> Result<(), Error> {     // static void rp1_pio_gpio_init(PIO pio, uint pin)
        self.gpio_set_function(pin, GPIO_FUNC_PIO)
    }

    ////////// Not in piolib, but buried in the example piolib/examples/rp1sm.c from https://github.com/raspberrypi/utils.

    pub fn read_hw(&self, addr: u32, data: &mut [u32]) -> Result<u32, Error> {
        let addr = 0xf000_0000 | addr; // proc_pio.h / proc-pio.rs register offsets don't include this base definition.
        let mut args = AccessHwArgs { addr,
                                      len: data.len() as u32 * size_of::<u32>() as u32,
                                      data: data as *mut [u32] as *mut c_void };
        self.rp1_ioctl_mut(PIO_IOC_READ_HW, &mut args)
    }

    pub fn write_hw(&self, addr: u32, data: &[u32]) -> Result<u32, Error> {
        let addr = 0xf000_0000 | addr; // proc_pio.h / proc-pio.rs register offsets don't include this base definition.
        let args = AccessHwArgs { addr, len: data.len() as u32, data: data as *const [u32] as *mut c_void };
        self.rp1_ioctl(PIO_IOC_WRITE_HW, &args)
    }
}


pub struct StateMachine<'a> {
    pio: &'a Rp1PIO,
    index: u16,
}

impl<'a> StateMachine<'a> {
    pub fn unclaim(self) -> Result<bool, Error> {
        let args = SmClaimArgs { mask: 1 << self.index };
        self.pio.rp1_ioctl(PIO_IOC_SM_UNCLAIM, &args)
            .map(|r| r != 0)
    }

    pub fn is_claimed(&self) -> Result<bool, Error> {
        let args = SmClaimArgs { mask: 1 << self.index };
        self.pio.rp1_ioctl(PIO_IOC_SM_IS_CLAIMED, &args)
            .map(|r| r > 0) // FIXME: when is this false?
    }

    pub fn init(&self, initial_pc: u16, config: &SmConfig) -> Result<(), Error> {
        if initial_pc >= INSTRUCTION_COUNT {
            Err(Error::BadPC { pc: initial_pc, max: INSTRUCTION_COUNT })?;
        }
        let args = SmInitArgs { sm: self.index, initial_pc, config: *config };
        self.pio.rp1_ioctl(PIO_IOC_SM_INIT, &args)
            .map(|_| ())
    }

    pub fn set_config(&self, config: &SmConfig) -> Result<(), Error> {
        let args = SmInitArgs { sm: self.index, initial_pc:0, config: *config };
        self.pio.rp1_ioctl(PIO_IOC_SM_SET_CONFIG, &args)
            .map(|_| ())
    }

    pub fn exec(&self, instr: u16, blocking: bool) -> Result<(), Error> {
        let args = SmExecArgs { sm: self.index, instr, blocking: blocking.into(), rsvd: 0 };
        self.pio.rp1_ioctl(PIO_IOC_SM_EXEC, &args)
            .map(|_| ())
    }

    pub fn clear_fifos(&self) -> Result<(), Error> {
        let args = SmClearFifosArgs { sm: self.index };
        self.pio.rp1_ioctl(PIO_IOC_SM_CLEAR_FIFOS, &args)
            .map(|_| ())
    }

    pub fn set_clkdiv_int_frac(&self, div: ClkDiv) -> Result<(), Error> {
        let args = SmSetClkdivArgs { sm: self.index, div_int: div.div, div_frac: div.frac, rsvd: 0 };
        self.pio.rp1_ioctl(PIO_IOC_SM_SET_CLKDIV, &args)
            .map(|_| ())
    }

    pub fn set_clkdiv(&self, div: f64) -> Result<(), Error> {
        self.set_clkdiv_int_frac(div.try_into()?)
    }

    pub fn set_pins(&self, pin_values: u32) -> Result<(), Error> {
        self.set_pins_with_mask(pin_values, GPIOS_MASK)
        // let args = SmSetPinsArgs { sm: self.index, values: pin_values, mask: GPIOS_MASK, rsvd:0 };
        // self.pio.rp1_ioctl(PIO_IOC_SM_SET_PINS, &args)
        //     .map(|_| ())
    }

    pub fn set_pins_with_mask(&self, pin_values: u32, pin_mask: u32) -> Result<(), Error> {
        let args = SmSetPinsArgs { sm: self.index, values: pin_values, mask: pin_mask, rsvd:0 };
        self.pio.rp1_ioctl(PIO_IOC_SM_SET_PINS, &args)
            .map(|_| ())
    }

    pub fn set_pindirs_with_mask(&self, pin_dirs: u32, pin_mask: u32) -> Result<(), Error> {
        if pin_dirs & GPIOS_MASK != pin_dirs {
            Err(Error::BadPinDirs(pin_dirs & !GPIOS_MASK))?;
        }
        if pin_mask & GPIOS_MASK != pin_mask {
            Err(Error::BadPinMask(pin_mask & !GPIOS_MASK))?;
        }
        let args = SmSetPindirsArgs { sm: self.index, dirs: pin_dirs, mask: pin_mask, rsvd:0 };
        self.pio.rp1_ioctl(PIO_IOC_SM_SET_PINDIRS, &args)
            .map(|_| ())
    }

    pub fn set_consecutive_pindirs(&self, pin_base: u32, pin_count:u32, is_out: bool) -> Result<(), Error> {
        let mask = ((1_u32 << pin_count) - 1) << pin_base;
        self.set_pindirs_with_mask(if is_out { mask } else { 0}, mask)
    }

    pub fn set_enabled(&self, enabled:bool) -> Result<(), Error> {
        self.pio.sm_set_enabled_mask(1 << self.index, enabled)
    }

    pub fn restart(&self) -> Result<(), Error> {
        self.pio.sm_restart_mask(1 << self.index)
    }

    pub fn clkdiv_restart(&self) -> Result<(), Error> {
        self.pio.sm_clkdiv_restart_mask(1 << self.index)
    }

    pub fn put(&self, data: u32, blocking: bool) -> Result<(), Error> {
        let args = SmPutArgs { sm: self.index, data, blocking: blocking.into(), rsvd:0 };
        self.pio.rp1_ioctl(PIO_IOC_SM_PUT, &args)
            .map(|_| ())
    }

    pub fn get(&self, blocking: bool) -> Result<u32, Error> {
        let mut args = SmGetArgs { sm: self.index, data:0, blocking: blocking.into(), rsvd:0 };
        self.pio.rp1_ioctl_mut(PIO_IOC_SM_GET, &mut args)?;
        Ok(args.data)
    }

    pub fn set_dmactrl(&self, is_tx:bool, ctrl: u32) -> Result<(), Error> {
        let args = SmSetDmactrlArgs { sm: self.index, is_tx: is_tx.into(), ctrl, rsvd:0 };
        self.pio.rp1_ioctl(PIO_IOC_SM_SET_DMACTRL, &args)
            .map(|_| ())
    }

    fn fifo_state(&self, tx: bool) -> Result<SmFifoStateArgs, Error> {
        let mut args = SmFifoStateArgs { sm: self.index, tx: tx.into(), level:0, empty:0, full:0, rsvd:0 };
        self.pio.rp1_ioctl_mut(PIO_IOC_SM_FIFO_STATE, &mut args)?;
        Ok(args)
    }

    pub fn is_rx_fifo_empty(&self) -> Result<bool, Error> {
        Ok(self.fifo_state(false)?.empty != 0)
    }

    pub fn is_rx_fifo_full(&self) -> Result<bool, Error> {
        Ok(self.fifo_state(false)?.full != 0)
    }

    pub fn get_rx_fifo_level(&self) -> Result<u16, Error> {
        Ok(self.fifo_state(false)?.level)
    }

    pub fn is_tx_fifo_empty(&self) -> Result<bool, Error> {
        Ok(self.fifo_state(true)?.empty != 0)
    }

    pub fn is_tx_fifo_full(&self) -> Result<bool, Error> {
        Ok(self.fifo_state(true)?.full != 0)
    }

    pub fn get_tx_fifo_level(&self) -> Result<u16, Error> {
        Ok(self.fifo_state(true)?.level)
    }

    pub fn drain_tx_fifo(&self) -> Result<(), Error> {
        let args = SmClearFifosArgs { sm: self.index };
        self.pio.rp1_ioctl(PIO_IOC_SM_DRAIN_TX, &args)
            .map(|_| ())
    }

    pub fn read_hw_state_machine(&self) -> Result<StateMachineHw, Error> {
        // Taken from piolib/examples/rp1sm.c in https://github.com/raspberrypi/utils
        let mut data = [0; 0x20];
        self.pio.read_hw(PROC_PIO_SM0_CLKDIV_OFFSET + self.index as u32 * 0x20, &mut data)?;
        let mut ctrl_data = [0; 1];
        self.pio.read_hw(PROC_PIO_CTRL_OFFSET, &mut ctrl_data)?;
        Ok(StateMachineHw {
            enabled    : (ctrl_data[0] >> self.index as u32) != 0,
            clkdiv     : data[0],
            execctrl   : data[1],
            shiftctrl  : data[2],
            pc         : data[3],
            instr      : data[4],
            pinctrl    : data[5],
            dmactrl_tx : data[6],
            dmactrl_rx : data[7],
        })
    }

    pub fn read_hw_fifo(&self) -> Result<FifoHw, Error> {
        // Taken from piolib/examples/rp1sm.c in https://github.com/raspberrypi/utils
        let mut data = [0; 64];
        self.pio.read_hw(PROC_PIO_CTRL_OFFSET, &mut data)?;
        let raw = RawFifoHw {
            ctrl    : data[0],
            fstat   : data[1],
            flevel  : data[3],
            flevel2 : data[4],
        };
        Ok(FifoHw {
            tx: FifoHwState {
                level: ((raw.flevel >> (self.index as u32 * 8)) & 0xf) + (((raw.flevel2 >> (self.index as u32 * 8)) & 1) << 4),
                full: raw.fstat & (0x10000 << self.index) != 0,
                empty: raw.fstat & (0x1000000 << self.index) != 0,
            },
            rx: FifoHwState {
                level: ((raw.flevel >> (self.index * 8 + 4)) & 0xf) + (((raw.flevel2 >> (self.index * 8 + 4)) & 1) << 4),
                full: raw.fstat & (0x1 << self.index) != 0,
                empty: raw.fstat & (0x100 << self.index) != 0,
            },
            raw,
        })
    }
}


#[repr(u16)]
pub enum XferDir {
    ToSm   = 0,
    FromSm = 1,
}


pub struct PioProgram {
    instructions: Vec<u16>,
    origin: i8,
    #[allow(dead_code)]
    pio_version: u8,
}

impl PioProgram {
    pub fn new(instructions: &[u16], origin: Option<u8>) -> PioProgram {
        PioProgram { instructions: instructions.to_owned(),
            origin: origin.map(|o| o as i8).unwrap_or(-1),
            pio_version: 0,
        }
    }
}


pub struct ClkDiv {
    pub div: u16,
    pub frac: u8,
}

impl TryFrom<f64> for ClkDiv {
    type Error=Error;

    fn try_from(div: f64) -> Result<Self, Self::Error> {
        if div != 0_f64 && (div < 1_f64 || div > 65536_f64) {
            Err(Error::BadDiv { div: div, min: 1_f64, max: 65536_f64 })?;
        }
        let div_int = div as u16;
        if div_int == 0 {
            Ok(ClkDiv { div: 0, frac: 0 })
        } else {
            Ok(ClkDiv { div: div_int, frac: (div.fract() * 256_f64) as u8 })
        }
    }
}


#[derive(Debug)]
pub struct StateMachineHw {
    pub enabled    : bool,
    pub clkdiv     : u32,
    pub execctrl   : u32,
    pub shiftctrl  : u32,
    pub pc         : u32,
    pub instr      : u32,
    pub pinctrl    : u32,
    pub dmactrl_tx : u32,
    pub dmactrl_rx : u32,
}

#[derive(Debug)]
pub struct RawFifoHw {
    pub ctrl    : u32,
    pub fstat   : u32,
    pub flevel  : u32,
    pub flevel2 : u32,
}

#[derive(Debug)]
pub struct FifoHw {
    pub raw: RawFifoHw,
    pub tx: FifoHwState,
    pub rx: FifoHwState,
}

#[derive(Debug)]
pub struct FifoHwState {
    pub level: u32,
    pub full: bool,
    pub empty: bool,
}

