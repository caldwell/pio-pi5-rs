// Copyright © 2025 David Caldwell <david@porkrind.org>
// SPDX-License-Identifier: BSD-3-Clause

#[repr(u16)]
pub enum Function {
    XIP  = 0,
    SPI  = 1,
    UART = 2,
    I2C  = 3,
    PWM  = 4,
    SIO  = 5,
    PIO0 = 6,
    PIO1 = 7,
    GPCK = 8,
    USB  = 9,
    NULL = 0x1f,
}

#[repr(u8)]
pub enum Direction {
    In = 0,
    Out = 1,
}

#[repr(u16)]
pub enum DriveStrength {
    /**< 2 mA nominal drive strength */  _2MA = 0,
    /**< 4 mA nominal drive strength */  _4MA = 1,
    /**< 8 mA nominal drive strength */  _8MA = 2,
    /**< 12 mA nominal drive strength */ _12MA = 3,
}
