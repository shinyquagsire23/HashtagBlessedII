/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

//#![allow(warnings, unused)]

use crate::util::*;

pub const CAR_PADDR: u32 = 0x60006000;

pub const PINMUX_PADDR: u32 = 0x70003000;
pub const PINMUX_VADDR: u32 = 0x70003000;

pub const PINMUX_AUX_UART1_BASE: u32 = (PINMUX_VADDR + 0xE4);
pub const PINMUX_AUX_UART1_TX_0: *mut u32 = (PINMUX_AUX_UART1_BASE + 0x0) as _;
pub const PINMUX_AUX_UART1_RX_0: *mut u32 = (PINMUX_AUX_UART1_BASE + 0x4) as _;
pub const PINMUX_AUX_UART1_RTS_0: *mut u32 = (PINMUX_AUX_UART1_BASE + 0x8) as _;
pub const PINMUX_AUX_UART1_CTS_0: *mut u32 = (PINMUX_AUX_UART1_BASE + 0xC) as _;

pub const PINMUX_AUX_UART2_BASE: u32 = (PINMUX_VADDR + 0xF4);
pub const PINMUX_AUX_UART2_TX_0: *mut u32 = (PINMUX_AUX_UART2_BASE + 0x0) as _;
pub const PINMUX_AUX_UART2_RX_0: *mut u32 = (PINMUX_AUX_UART2_BASE + 0x4) as _;
pub const PINMUX_AUX_UART2_RTS_0: *mut u32 = (PINMUX_AUX_UART2_BASE + 0x8) as _;
pub const PINMUX_AUX_UART2_CTS_0: *mut u32 = (PINMUX_AUX_UART2_BASE + 0xC) as _;

pub const PINMUX_AUX_UART3_BASE: u32 = (PINMUX_VADDR + 0x104);
pub const PINMUX_AUX_UART3_TX_0: *mut u32 = (PINMUX_AUX_UART3_BASE + 0x0) as _;
pub const PINMUX_AUX_UART3_RX_0: *mut u32 = (PINMUX_AUX_UART3_BASE + 0x4) as _;
pub const PINMUX_AUX_UART3_RTS_0: *mut u32 = (PINMUX_AUX_UART3_BASE + 0x8) as _;
pub const PINMUX_AUX_UART3_CTS_0: *mut u32 = (PINMUX_AUX_UART3_BASE + 0xC) as _;

pub const PINMUX_AUX_UART4_BASE: u32 = (PINMUX_VADDR + 0x114);
pub const PINMUX_AUX_UART4_TX_0: *mut u32 = (PINMUX_AUX_UART4_BASE + 0x0) as _;
pub const PINMUX_AUX_UART4_RX_0: *mut u32 = (PINMUX_AUX_UART4_BASE + 0x4) as _;
pub const PINMUX_AUX_UART4_RTS_0: *mut u32 = (PINMUX_AUX_UART4_BASE + 0x8) as _;
pub const PINMUX_AUX_UART4_CTS_0: *mut u32 = (PINMUX_AUX_UART4_BASE + 0xC) as _;

pub const PINMUX_SCHMT:     u32 = bit!(12);
pub const PINMUX_LPDR:      u32 = bit!(8);
pub const PINMUX_LOCK:      u32 = bit!(7);
pub const PINMUX_INPUT:     u32 = bit!(6);
pub const PINMUX_PARKED:    u32 = bit!(5);
pub const PINMUX_TRISTATE:  u32 = bit!(4);

pub const PINMUX_PM:        u32 = 0b11;
pub const PINMUX_PULL_DOWN: u32 = bit!(2);
pub const PINMUX_PULL_UP:   u32 = bit!(3);

pub const PINMUX_DRIVE: u32 = 0b11 << 13;
pub const PINMUX_DRIVE_2X: u32 = 1 << 13;

pub const PINMUX_RSVD2: u32 = 2;

lazy_static! {
    static ref PINMUX_REG: MMIOReg = MMIOReg::new(PINMUX_AUX_UART4_BASE);
}
