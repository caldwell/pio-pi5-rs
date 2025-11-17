// Copyright © 2025 David Caldwell <david@porkrind.org>
// SPDX-License-Identifier: BSD-3-Clause

use libc::{c_ulong,_IOW,_IO,_IOWR};

use crate::INSTRUCTION_COUNT;

#[repr(C)]
#[derive(Clone,Copy)]
pub struct SmConfig {
    clkdiv:     u32,
    execctrl:   u32,
    shiftctrl:  u32,
    pinctrl:    u32,
}

#[repr(C)]
pub(crate) struct AddProgramArgs {
    pub(crate) num_instrs: u16,
    pub(crate) origin:     u16,
    pub(crate) instrs:     [u16; INSTRUCTION_COUNT as usize],
}

#[repr(C)]
pub(crate) struct RemoveProgramArgs {
    pub(crate) num_instrs:  u16,
    pub(crate) origin:      u16,
}

#[repr(C)]
pub(crate) struct SmClaimArgs {
    pub(crate) mask: u16,
}

#[repr(C)]
pub(crate) struct SmInitArgs {
    pub(crate) sm:         u16,
    pub(crate) initial_pc: u16,
    pub(crate) config:     SmConfig,
}

#[repr(C)]
pub(crate) struct SmSetConfigArgs {
    pub(crate) sm:     u16,
    pub(crate) rsvd:   u16,
    pub(crate) config: SmConfig,
}

#[repr(C)]
pub(crate) struct SmExecArgs {
    pub(crate) sm:        u16,
    pub(crate) instr:     u16,
    pub(crate) blocking:  u8,
    pub(crate) rsvd:      u8,
}


#[repr(C)]
pub(crate) struct SmClearFifosArgs {
    pub(crate) sm: u16,
}


#[repr(C)]
pub(crate) struct SmSetClkdivArgs {
    pub(crate) sm:        u16,
    pub(crate) div_int:   u16,
    pub(crate) div_frac:  u8,
    pub(crate) rsvd:      u8,
}


#[repr(C)]
pub(crate) struct SmSetPinsArgs {
    pub(crate) sm:      u16,
    pub(crate) rsvd:    u16,
    pub(crate) values:  u32,
    pub(crate) mask:    u32,
}


#[repr(C)]
pub(crate) struct SmSetPindirsArgs {
    pub(crate) sm:    u16,
    pub(crate) rsvd:  u16,
    pub(crate) dirs:  u32,
    pub(crate) mask:  u32,
}


#[repr(C)]
pub(crate) struct SmSetEnabledArgs {
    pub(crate) mask:    u16,
    pub(crate) enable:  u8,
    pub(crate) rsvd:    u8,
}


#[repr(C)]
pub(crate) struct SmRestartArgs {
    pub(crate) mask: u16,
}


#[repr(C)]
pub(crate) struct SmClkdivRestartArgs {
    pub(crate) mask: u16,
}


#[repr(C)]
pub(crate) struct SmEnableSyncArgs {
    pub(crate) mask: u16,
}


#[repr(C)]
pub(crate) struct SmPutArgs {
    pub(crate) sm:        u16,
    pub(crate) blocking:  u8,
    pub(crate) rsvd:      u8,
    pub(crate) data:      u32,
}


#[repr(C)]
pub(crate) struct SmGetArgs {
    pub(crate) sm:        u16,
    pub(crate) blocking:  u8,
    pub(crate) rsvd:      u8,
    pub(crate) data:      u32, // OUT
}


#[repr(C)]
pub(crate) struct SmSetDmactrlArgs {
    pub(crate) sm:     u16,
    pub(crate) is_tx:  u8,
    pub(crate) rsvd:   u8,
    pub(crate) ctrl:   u32,
}


#[repr(C)]
pub(crate) struct SmFifoStateArgs {
    pub(crate) sm:     u16,
    pub(crate) tx:     u8,
    pub(crate) rsvd:   u8,
    pub(crate) level:  u16, // OUT
    pub(crate) empty:  u8,  // OUT
    pub(crate) full:   u8,   // OUT
}


#[repr(C)]
pub(crate) struct GpioInitArgs {
    pub(crate) gpio: u16,
}


#[repr(C)]
pub(crate) struct GpioSetFunctionArgs {
    pub(crate) gpio: u16,
    pub(crate) func: u16,
}


#[repr(C)]
pub(crate) struct GpioSetPullsArgs {
    pub(crate) gpio:  u16,
    pub(crate) up:    u8,
    pub(crate) down:  u8,
}


#[repr(C)]
pub(crate) struct GpioSetArgs {
    pub(crate) gpio:   u16,
    pub(crate) value:  u16,
}


#[repr(C)]
pub(crate) struct SmConfigXferArgs {
    pub(crate) sm:         u16,
    pub(crate) dir:        u16,
    pub(crate) buf_size:   u16,
    pub(crate) buf_count:  u16,
}


#[repr(C)]
pub(crate) struct SmConfigXfer32Args {
    pub(crate) sm:         u16,
    pub(crate) dir:        u16,
    pub(crate) buf_size:   u32,
    pub(crate) buf_count:  u32,
}


#[repr(C)]
pub(crate) struct SmXferDataArgs {
    pub(crate) sm:          u16,
    pub(crate) dir:         u16,
    pub(crate) data_bytes:  u16,
    pub(crate) rsvd:        u16,
    pub(crate) data:        *const std::ffi::c_void,
}


#[repr(C)]
pub(crate) struct SmXferData32Args {
    pub(crate) sm:          u16,
    pub(crate) dir:         u16,
    pub(crate) data_bytes:  u32,
    pub(crate) data:        *const std::ffi::c_void,
}


#[repr(C)]
pub(crate) struct AccessHwArgs {
    pub(crate) addr:  u32,
    pub(crate) len:   u32,
    pub(crate) data:  *const std::ffi::c_void,
}


const PIO_IOC_MAGIC: u32 = 102;

pub(crate) const PIO_IOC_SM_CONFIG_XFER         : c_ulong = _IOW::<SmConfigXferArgs> (PIO_IOC_MAGIC, 0);
pub(crate) const PIO_IOC_SM_XFER_DATA           : c_ulong = _IOW::<SmXferDataArgs> (PIO_IOC_MAGIC, 1/*, struct SmXferDataArgs*/);
pub(crate) const PIO_IOC_SM_XFER_DATA32         : c_ulong = _IOW::<SmXferData32Args> (PIO_IOC_MAGIC, 2/*, struct SmXferData32Args*/);
pub(crate) const PIO_IOC_SM_CONFIG_XFER32       : c_ulong = _IOW::<SmConfigXfer32Args> (PIO_IOC_MAGIC, 3/*, struct SmConfigXfer32Args*/);

pub(crate) const PIO_IOC_READ_HW                : c_ulong = _IOW::<AccessHwArgs> (PIO_IOC_MAGIC, 8/*, struct Rp1AccessHwArgs*/);
pub(crate) const PIO_IOC_WRITE_HW               : c_ulong = _IOW::<AccessHwArgs> (PIO_IOC_MAGIC, 9/*, struct Rp1AccessHwArgs*/);

pub(crate) const PIO_IOC_CAN_ADD_PROGRAM        : c_ulong = _IOW::<AddProgramArgs> (PIO_IOC_MAGIC, 10/*, struct AddProgramArgs*/);
pub(crate) const PIO_IOC_ADD_PROGRAM            : c_ulong = _IOW::<AddProgramArgs> (PIO_IOC_MAGIC, 11/*, struct AddProgramArgs*/);
pub(crate) const PIO_IOC_REMOVE_PROGRAM         : c_ulong = _IOW::<RemoveProgramArgs> (PIO_IOC_MAGIC, 12/*, struct RemoveProgramArgs*/);
pub(crate) const PIO_IOC_CLEAR_INSTR_MEM        : c_ulong = _IO::<>  (PIO_IOC_MAGIC, 13/**/);

pub(crate) const PIO_IOC_SM_CLAIM               : c_ulong = _IOW::<SmClaimArgs> (PIO_IOC_MAGIC, 20/*, struct SmClaimArgs*/);
pub(crate) const PIO_IOC_SM_UNCLAIM             : c_ulong = _IOW::<SmClaimArgs> (PIO_IOC_MAGIC, 21/*, struct SmClaimArgs*/);
pub(crate) const PIO_IOC_SM_IS_CLAIMED          : c_ulong = _IOW::<SmClaimArgs> (PIO_IOC_MAGIC, 22/*, struct SmClaimArgs*/);

pub(crate) const PIO_IOC_SM_INIT                : c_ulong = _IOW::<SmInitArgs> (PIO_IOC_MAGIC, 30/*, struct SmInitArgs*/);
pub(crate) const PIO_IOC_SM_SET_CONFIG          : c_ulong = _IOW::<SmSetConfigArgs> (PIO_IOC_MAGIC, 31/*, struct SmSetConfigArgs*/);
pub(crate) const PIO_IOC_SM_EXEC                : c_ulong = _IOW::<SmExecArgs> (PIO_IOC_MAGIC, 32/*, struct SmExecArgs*/);
pub(crate) const PIO_IOC_SM_CLEAR_FIFOS         : c_ulong = _IOW::<SmClearFifosArgs> (PIO_IOC_MAGIC, 33/*, struct SmClearFifosArgs*/);
pub(crate) const PIO_IOC_SM_SET_CLKDIV          : c_ulong = _IOW::<SmSetClkdivArgs> (PIO_IOC_MAGIC, 34/*, struct SmSetClkdivArgs*/);
pub(crate) const PIO_IOC_SM_SET_PINS            : c_ulong = _IOW::<SmSetPinsArgs> (PIO_IOC_MAGIC, 35/*, struct SmSetPinsArgs*/);
pub(crate) const PIO_IOC_SM_SET_PINDIRS         : c_ulong = _IOW::<SmSetPindirsArgs> (PIO_IOC_MAGIC, 36/*, struct SmSetPindirsArgs*/);
pub(crate) const PIO_IOC_SM_SET_ENABLED         : c_ulong = _IOW::<SmSetEnabledArgs> (PIO_IOC_MAGIC, 37/*, struct SmSetEnabledArgs*/);
pub(crate) const PIO_IOC_SM_RESTART             : c_ulong = _IOW::<SmRestartArgs> (PIO_IOC_MAGIC, 38/*, struct SmRestartArgs*/);
pub(crate) const PIO_IOC_SM_CLKDIV_RESTART      : c_ulong = _IOW::<SmRestartArgs> (PIO_IOC_MAGIC, 39/*, struct SmRestartArgs*/);
pub(crate) const PIO_IOC_SM_ENABLE_SYNC         : c_ulong = _IOW::<SmEnableSyncArgs> (PIO_IOC_MAGIC, 40/*, struct SmEnableSyncArgs*/);
pub(crate) const PIO_IOC_SM_PUT                 : c_ulong = _IOW::<SmPutArgs> (PIO_IOC_MAGIC, 41/*, struct SmPutArgs*/);
pub(crate) const PIO_IOC_SM_GET                 : c_ulong = _IOWR::<SmGetArgs>(PIO_IOC_MAGIC, 42/*, struct SmGetArgs*/);
pub(crate) const PIO_IOC_SM_SET_DMACTRL         : c_ulong = _IOW::<SmSetDmactrlArgs> (PIO_IOC_MAGIC, 43/*, struct SmSetDmactrlArgs*/);
pub(crate) const PIO_IOC_SM_FIFO_STATE          : c_ulong = _IOW::<SmFifoStateArgs> (PIO_IOC_MAGIC, 44/*, struct SmFifoStateArgs*/); // FIXME: Should be _IOWR???
pub(crate) const PIO_IOC_SM_DRAIN_TX            : c_ulong = _IOW::<SmClearFifosArgs> (PIO_IOC_MAGIC, 45/*, struct SmClearFifosArgs*/);

pub(crate) const PIO_IOC_GPIO_INIT              : c_ulong = _IOW::<GpioInitArgs> (PIO_IOC_MAGIC, 50/*, struct Rp1GpioInitArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_FUNCTION      : c_ulong = _IOW::<GpioSetFunctionArgs> (PIO_IOC_MAGIC, 51/*, struct Rp1GpioSetFunctionArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_PULLS         : c_ulong = _IOW::<GpioSetPullsArgs> (PIO_IOC_MAGIC, 52/*, struct Rp1GpioSetPullsArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_OUTOVER       : c_ulong = _IOW::<GpioSetArgs> (PIO_IOC_MAGIC, 53/*, struct Rp1GpioSetArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_INOVER        : c_ulong = _IOW::<GpioSetArgs> (PIO_IOC_MAGIC, 54/*, struct Rp1GpioSetArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_OEOVER        : c_ulong = _IOW::<GpioSetArgs> (PIO_IOC_MAGIC, 55/*, struct Rp1GpioSetArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_INPUT_ENABLED : c_ulong = _IOW::<GpioSetArgs> (PIO_IOC_MAGIC, 56/*, struct Rp1GpioSetArgs*/);
pub(crate) const PIO_IOC_GPIO_SET_DRIVE_STRENGTH: c_ulong = _IOW::<GpioSetArgs> (PIO_IOC_MAGIC, 57/*, struct Rp1GpioSetArgs*/);



