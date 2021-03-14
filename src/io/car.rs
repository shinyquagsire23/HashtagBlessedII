/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
#![allow(warnings, unused)]

use crate::util::*;

const CAR_PADDR: u32 = 0x60006000;
const CAR_VADDR: u32 = 0x60006000;

pub struct CarDeviceInfo
{
    rst_dev_offset: u32,
    clk_out_enb_offset: u32,
    clk_source_offset: u32,
    dev_bit: u8,
    clk_source: u8,
    clk_divisor: u8,
}

pub const CAR_INFO_UART_A: CarDeviceInfo = CarDeviceInfo {
    rst_dev_offset: 0x004,
    clk_out_enb_offset: 0x010,
    clk_source_offset: 0x178,
    dev_bit: 6,
    clk_source: 0,
    clk_divisor: 0,
};

pub const CAR_INFO_UART_B: CarDeviceInfo = CarDeviceInfo {
    rst_dev_offset: 0x004,
    clk_out_enb_offset: 0x010,
    clk_source_offset: 0x17C,
    dev_bit: 7,
    clk_source: 0,
    clk_divisor: 0,
};

pub const CAR_INFO_UART_C: CarDeviceInfo = CarDeviceInfo {
    rst_dev_offset: 0x008,
    clk_out_enb_offset: 0x014,
    clk_source_offset: 0x1A0,
    dev_bit: 23,
    clk_source: 0,
    clk_divisor: 0,
};

pub const CAR_INFO_UART_D: CarDeviceInfo = CarDeviceInfo {
    rst_dev_offset: 0x00C,
    clk_out_enb_offset: 0x0148,
    clk_source_offset: 0x1C0,
    dev_bit: 1,
    clk_source: 0,
    clk_divisor: 0,
};

impl CarDeviceInfo
{
    pub fn enable(&self) {

        self.disable();

        let rst_dev_reg: *mut u32 = (CAR_VADDR + self.rst_dev_offset) as _;
        let clk_enb_reg: *mut u32 = (CAR_VADDR + self.clk_out_enb_offset) as _;
        let clk_src_reg: *mut u32 = (CAR_VADDR + self.clk_source_offset) as _;

        unsafe
        {
            if(self.clk_source_offset != 0)
            {
                clk_src_reg.write_volatile((self.clk_source as u32) << 29 | (self.clk_divisor as u32));
            }
        
            let prev_enb = clk_enb_reg.read_volatile();
            let prev_dev = rst_dev_reg.read_volatile();
            clk_enb_reg.write_volatile(prev_dev | bit!(self.dev_bit));
            rst_dev_reg.write_volatile(prev_dev & !bit!(self.dev_bit));
        }
    }
    
    pub fn disable(&self) {
        let rst_dev_reg: *mut u32 = (CAR_VADDR + self.rst_dev_offset) as _;
        let clk_enb_reg: *mut u32 = (CAR_VADDR + self.clk_out_enb_offset) as _;

        unsafe
        {
            let prev_dev = rst_dev_reg.read_volatile();
            let prev_enb = clk_enb_reg.read_volatile();
            rst_dev_reg.write_volatile(prev_dev | bit!(self.dev_bit));
            clk_enb_reg.write_volatile(prev_dev & !bit!(self.dev_bit));
        }
    }
    
    pub fn isEnabled(&self) -> bool {
        let rst_dev_reg: *mut u32 = (CAR_VADDR + self.rst_dev_offset) as _;
        let clk_enb_reg: *mut u32 = (CAR_VADDR + self.clk_out_enb_offset) as _;

        unsafe
        {
            let rst_dev = rst_dev_reg.read_volatile();
            let clk_enb = clk_enb_reg.read_volatile();
            
            if (rst_dev & bit!(self.dev_bit) != 0) {
                return false;
            }

            if (clk_enb & bit!(self.dev_bit) == 0) {
                return false;
            }

            return true;
        }
    }
}
