// Copyright © 2025 David Caldwell <david@porkrind.org>
// SPDX-License-Identifier: BSD-3-Clause

mod config;
pub mod gpio;
mod ioctl;
#[path="proc-pio.rs"]
pub mod proc_pio;
mod rp1pio;

pub use self::rp1pio::*;
pub use self::config::SmConfig;

use std::sync::{LazyLock, Mutex};

use crate::gpio::Function;

const INSTRUCTION_COUNT  : u16 = 32;
const SM_COUNT           : u16 =  4;
const GPIO_COUNT         : usize = 28;
const GPIOS_MASK         : u32 = (1 << GPIO_COUNT) - 1;
const GPIO_FUNC_PIO      : Function = Function::PIO1; // function 7

#[derive(Clone)]
pub struct Chip {
    pub name: String,
    pub compatible: String,
    pub instr_count: u16,
    pub sm_count: u16,
    pub fifo_depth: u16,
}

impl Chip {
    pub fn new() -> Chip {
        Chip { // Values taken from piolib/pio_rp1.c
            name: "rp1".to_string(),
            compatible: "raspberrypi,rp1-pio".to_string(),
            instr_count: INSTRUCTION_COUNT,
            sm_count: SM_COUNT,
            fifo_depth: 8,
        }
    }
}

#[derive(Clone)]
struct PIOReservation {
    chip: Chip,
    in_use: bool,
}

struct PIOInstance {
    chip: Chip,
    index: usize,
}

static INSTANCES: LazyLock<Mutex<Vec<PIOReservation>>> = LazyLock::new(|| {
    Mutex::new(PIOReservation::reservations())
});

impl PIOReservation {
    fn reservations() -> Vec<PIOReservation> {
        // Right now the RP1 only has a single PIO available for use.
        vec![PIOReservation { chip: Chip::new(), in_use: false }]
    }
}

impl PIOInstance {
    fn reserve(index: usize) -> Result<PIOInstance, Error> {
        let chip = {
            let mut instances = INSTANCES.lock().unwrap();
            let Some(instance) = instances.get_mut(index) else {
                return Err(Error::BadPIOInstance { index, max: instances.len() });
            };
            if instance.in_use {
                return Err(Error::InstanceInUse);
            }
            instance.in_use = true;
            instance.chip.clone()
        };
        Ok(PIOInstance { chip, index })
    }
}

impl Drop for PIOInstance {
    fn drop(&mut self) {
        // TODO: Can this deadlock with the above somehow? Think it through!
        let mut instances = INSTANCES.lock().unwrap();
        let instance = instances.get_mut(self.index).expect(&format!("Bad index in reserved PIO Instance: {}!", self.index));
        instance.in_use = false;
    }
}

#[derive(Debug)]
pub enum Error {
    BadPIOInstance { index: usize, max: usize },
    InstanceInUse,
    RemoteIOErr,
    TimedOut,
    IOError(std::io::Error),
    Unknown(i32),
    BadSM { sm:u16, max:u16 },
    BadSMMask { sm_mask:u16, max:u16 },
    OffsetOriginMismatch { origin: u8, offset: u16 },
    OffsetTooLarge { offset: u16, max: u16 },
    TooManyInstructions { instructions: usize, max: u16 },
    BadPC { pc: u16, max: u16 },
    BadDiv { div: f64, min: f64, max: f64 },
    BadPinDirs(u32),
    BadPinMask(u32),
    BadGPIO { gpio: u16, max: usize },
    ParamErr { param: &'static str, should_be: String },
}

impl std::error::Error for Error {
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadPIOInstance { index, max }             => write!(f, "Bad PIO Instance: {index} must be less than {max}"),
            Error::InstanceInUse                             => write!(f, "PIO Instance is in use"),
            Error::RemoteIOErr                               => write!(f, "Remote IO Error"),
            Error::TimedOut                                  => write!(f, "Timed Out"),
            Error::IOError(error)                            => write!(f, "IOError: {error}"),
            Error::Unknown(code)                             => write!(f, "Unknown Error Code {code} ({code:#x})"),
            Error::BadSM { sm, max }                         => write!(f, "Bad State Machine Index: {sm} must be less than {max}"),
            Error::BadSMMask { sm_mask, max }                => write!(f, "Bad State Machine Mask {sm_mask:b}: bits must be less than {max}"),
            Error::OffsetOriginMismatch { origin, offset }   => write!(f, "Offset/Origin Mismatch: {offset} != {origin}"),
            Error::OffsetTooLarge { offset, max }            => write!(f, "Offset Too Large: {offset} must be less than {max}"),
            Error::TooManyInstructions { instructions, max } => write!(f, "Too Many Instructions: {instructions} must be less than {max}"),
            Error::BadPC { pc, max }                         => write!(f, "Bad PC: {pc} must be less than {max}"),
            Error::BadDiv { div, min, max }                  => write!(f, "Bad Divider: {div} must be in {min}..={max}"),
            Error::BadPinDirs(pin_dirs)                      => write!(f, "Bad pin_dirs: The bits {pin_dirs:#b} are out of range"),
            Error::BadPinMask(pin_mask)                      => write!(f, "Bad pin_dirs: The bits {pin_mask:#b} are out of range"),
            Error::BadGPIO { gpio, max }                     => write!(f, "Bad GPIO: {gpio} must be less than {max}"),
            Error::ParamErr {param, should_be }              => write!(f, "Bad Parameter \"{param}\": should be {should_be}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value)
    }
}

#[repr(u32)]
pub enum PioFifoJoin {
    None = 0,
    Tx   = 1,
    Rx   = 2,
}

#[repr(u32)]
pub enum PioMovStatus {
    TxLessThan = 0,
    RxLessThan = 1,
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {
        todo!()
    }
}
