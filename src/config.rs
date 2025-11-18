// Copyright © 2025 David Caldwell <david@porkrind.org>

use crate::{proc_pio::*, ClkDiv, Error, PioFifoJoin, PioMovStatus, GPIO_COUNT, INSTRUCTION_COUNT};

#[repr(C)]
#[derive(Clone,Copy)]
pub struct SmConfig {
    clkdiv:     u32,
    execctrl:   u32,
    shiftctrl:  u32,
    pinctrl:    u32,
}

impl Default for SmConfig {
    fn default() -> Self {
        SmConfig { clkdiv: 0, execctrl: 0, shiftctrl: 0, pinctrl: 0 }
            .set_clkdiv(1.0).unwrap()
            .set_wrap(0, 31).unwrap()
            .set_in_shift(true, false, 32).unwrap()
            .set_out_shift(true, false, 32).unwrap()
    }
}

fn bool_to_u32(b: bool) -> u32 {
    b.into()
}

macro_rules! valid_params_if {
    [ $test:expr, $param:expr, $should_be:expr] => {
        if $test { Ok(()) }
        else { Err(Error::ParamErr { param: $param, should_be: $should_be }) }
    };
    [ $test:expr, $param:expr] => {
        if $test { Ok(()) }
        else { Err(Error::ParamErr { param: $param, should_be: stringify!($test).to_string() }) }
    };
}

impl SmConfig {
    pub fn set_out_pins(mut self, out_base: u32, out_count: u32) -> Result<Self, Error> {
        valid_params_if!(out_base < GPIO_COUNT as u32, "out_base", format!("< {GPIO_COUNT}"))?;
        valid_params_if!(out_base < GPIO_COUNT as u32, "out_base")?;
        self.pinctrl = (self.pinctrl & !(PROC_PIO_SM0_PINCTRL_OUT_BASE_BITS | PROC_PIO_SM0_PINCTRL_OUT_COUNT_BITS)) |
                       (out_base << PROC_PIO_SM0_PINCTRL_OUT_BASE_LSB) |
                       (out_count << PROC_PIO_SM0_PINCTRL_OUT_COUNT_LSB);
        Ok(self)
    }

    pub fn set_set_pins(mut self, set_base: u32, set_count: u32) -> Result<Self, Error> {
        valid_params_if!(set_base < GPIO_COUNT as u32, "set_base", format!("< {GPIO_COUNT}"))?;
        valid_params_if!(set_count <= 5, "set_count")?;
        self.pinctrl = (self.pinctrl & !(PROC_PIO_SM0_PINCTRL_SET_BASE_BITS | PROC_PIO_SM0_PINCTRL_SET_COUNT_BITS)) |
                       (set_base << PROC_PIO_SM0_PINCTRL_SET_BASE_LSB) |
                       (set_count << PROC_PIO_SM0_PINCTRL_SET_COUNT_LSB);
        Ok(self)
    }

    pub fn set_in_pins(mut self, in_base: u32) -> Result<Self, Error> {
        valid_params_if!(in_base < GPIO_COUNT as u32, "in_base", format!("< {GPIO_COUNT}"))?;
        self.pinctrl = (self.pinctrl & !PROC_PIO_SM0_PINCTRL_IN_BASE_BITS) |
                       (in_base << PROC_PIO_SM0_PINCTRL_IN_BASE_LSB);
        Ok(self)
    }

    pub fn set_sideset_pins(mut self, sideset_base: u32) -> Result<Self, Error> {
        valid_params_if!(sideset_base < GPIO_COUNT as u32, "sideset_base", format!("< {GPIO_COUNT}"))?;
        self.pinctrl = (self.pinctrl & !PROC_PIO_SM0_PINCTRL_SIDESET_BASE_BITS) |
                       (sideset_base << PROC_PIO_SM0_PINCTRL_SIDESET_BASE_LSB);
        Ok(self)
    }

    pub fn set_sideset(mut self, bit_count: u32, optional: bool, pindirs: bool) -> Result<Self, Error> {
        valid_params_if!(bit_count <= 5, "bit_count")?;
        valid_params_if!(!optional || bit_count >= 1, "option,bit_count")?;
        self.pinctrl = (self.pinctrl & !PROC_PIO_SM0_PINCTRL_SIDESET_COUNT_BITS) |
                       (bit_count << PROC_PIO_SM0_PINCTRL_SIDESET_COUNT_LSB);

        self.execctrl = (self.execctrl & !(PROC_PIO_SM0_EXECCTRL_SIDE_EN_BITS | PROC_PIO_SM0_EXECCTRL_SIDE_PINDIR_BITS)) |
                        (<bool as Into<u32>>::into(optional) << PROC_PIO_SM0_EXECCTRL_SIDE_EN_LSB) |
                        (<bool as Into<u32>>::into(pindirs) << PROC_PIO_SM0_EXECCTRL_SIDE_PINDIR_LSB);
        Ok(self)
    }

    pub fn set_clkdiv_int_frac(mut self, div: ClkDiv) -> Result<Self, Error> {
        self.clkdiv =
                ((div.frac as u32) << PROC_PIO_SM0_CLKDIV_FRAC_LSB) |
                ((div.div as u32) << PROC_PIO_SM0_CLKDIV_INT_LSB);
        Ok(self)
    }

    pub fn set_clkdiv(self, div: f64) -> Result<Self, Error> {
        self.set_clkdiv_int_frac(div.try_into()?)
    }

    pub fn set_wrap(mut self, wrap_target: u32, wrap: u32) -> Result<Self, Error> {
        valid_params_if!(wrap        < INSTRUCTION_COUNT as u32, "wrap",        format!("< {INSTRUCTION_COUNT}"))?;
        valid_params_if!(wrap_target < INSTRUCTION_COUNT as u32, "wrap_target", format!("< {INSTRUCTION_COUNT}"))?;
        self.execctrl = (self.execctrl & !(PROC_PIO_SM0_EXECCTRL_WRAP_TOP_BITS | PROC_PIO_SM0_EXECCTRL_WRAP_BOTTOM_BITS)) |
                        (wrap_target << PROC_PIO_SM0_EXECCTRL_WRAP_BOTTOM_LSB) |
                        (wrap << PROC_PIO_SM0_EXECCTRL_WRAP_TOP_LSB);
        Ok(self)
    }

    pub fn set_jmp_pin(mut self, pin: u32) -> Result<Self, Error> {
        valid_params_if!(pin < GPIO_COUNT as u32, "pin", format!("< {GPIO_COUNT}"))?;
        self.execctrl = (self.execctrl & !PROC_PIO_SM0_EXECCTRL_JMP_PIN_BITS) |
                        (pin << PROC_PIO_SM0_EXECCTRL_JMP_PIN_LSB);
        Ok(self)
    }

    pub fn set_in_shift(mut self, shift_right: bool, autopush: bool, push_threshold: u32) -> Result<Self, Error> {
        valid_params_if!(push_threshold <= 32, "push_threshold")?;
        self.shiftctrl = (self.shiftctrl &
                          !(PROC_PIO_SM0_SHIFTCTRL_IN_SHIFTDIR_BITS |
                            PROC_PIO_SM0_SHIFTCTRL_AUTOPUSH_BITS |
                            PROC_PIO_SM0_SHIFTCTRL_PUSH_THRESH_BITS)) |
                         (bool_to_u32(shift_right) << PROC_PIO_SM0_SHIFTCTRL_IN_SHIFTDIR_LSB) |
                         (bool_to_u32(autopush) << PROC_PIO_SM0_SHIFTCTRL_AUTOPUSH_LSB) |
                         ((push_threshold & 0x1f) << PROC_PIO_SM0_SHIFTCTRL_PUSH_THRESH_LSB);
        Ok(self)
    }

    pub fn set_out_shift(mut self, shift_right: bool, autopull: bool, pull_threshold: u32) -> Result<Self, Error> {
        valid_params_if!(pull_threshold <= 32, "pull_threshold")?;
        self.shiftctrl = (self.shiftctrl &
                          !(PROC_PIO_SM0_SHIFTCTRL_OUT_SHIFTDIR_BITS |
                            PROC_PIO_SM0_SHIFTCTRL_AUTOPULL_BITS |
                            PROC_PIO_SM0_SHIFTCTRL_PULL_THRESH_BITS)) |
                         (bool_to_u32(shift_right) << PROC_PIO_SM0_SHIFTCTRL_OUT_SHIFTDIR_LSB) |
                         (bool_to_u32(autopull) << PROC_PIO_SM0_SHIFTCTRL_AUTOPULL_LSB) |
                         ((pull_threshold & 0x1f) << PROC_PIO_SM0_SHIFTCTRL_PULL_THRESH_LSB);
        Ok(self)
    }

    pub fn set_fifo_join(mut self, join: PioFifoJoin) -> Result<Self, Error> {
        self.shiftctrl = (self.shiftctrl & !(PROC_PIO_SM0_SHIFTCTRL_FJOIN_TX_BITS | PROC_PIO_SM0_SHIFTCTRL_FJOIN_RX_BITS)) |
                         ((join as u32) << PROC_PIO_SM0_SHIFTCTRL_FJOIN_TX_LSB);
        Ok(self)
    }

    pub fn set_out_special(mut self, sticky: bool, has_enable_pin: bool, enable_pin_index: u32) -> Result<Self, Error> {
        self.execctrl = (self.execctrl &
                         !(PROC_PIO_SM0_EXECCTRL_OUT_STICKY_BITS | PROC_PIO_SM0_EXECCTRL_INLINE_OUT_EN_BITS |
                           PROC_PIO_SM0_EXECCTRL_OUT_EN_SEL_BITS)) |
                        (bool_to_u32(sticky) << PROC_PIO_SM0_EXECCTRL_OUT_STICKY_LSB) |
                        (bool_to_u32(has_enable_pin) << PROC_PIO_SM0_EXECCTRL_INLINE_OUT_EN_LSB) |
                        ((enable_pin_index << PROC_PIO_SM0_EXECCTRL_OUT_EN_SEL_LSB) & PROC_PIO_SM0_EXECCTRL_OUT_EN_SEL_BITS);
        Ok(self)
    }

    pub fn set_mov_status(mut self, status_sel: PioMovStatus, status_n: u32) -> Result<Self, Error> {
        self.execctrl = (self.execctrl &
                         !(PROC_PIO_SM0_EXECCTRL_STATUS_SEL_BITS | PROC_PIO_SM0_EXECCTRL_STATUS_N_BITS)) |
                        (((status_sel as u32) << PROC_PIO_SM0_EXECCTRL_STATUS_SEL_LSB) & PROC_PIO_SM0_EXECCTRL_STATUS_SEL_BITS) |
                        ((status_n << PROC_PIO_SM0_EXECCTRL_STATUS_N_LSB) & PROC_PIO_SM0_EXECCTRL_STATUS_N_BITS);
        Ok(self)
    }
}
