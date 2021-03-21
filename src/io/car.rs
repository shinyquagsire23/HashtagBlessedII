/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
#![allow(warnings, unused)]

use crate::util::*;

const CAR_PADDR: u32 = 0x60006000;
const CAR_VADDR: u32 = 0x60006000;


pub const CLK_RST_CONTROLLER_CLK_OUT_ENB_L:            u32 = (CAR_PADDR + 0x010);
pub const CLK_RST_CONTROLLER_OSC_CTRL:                 u32 = (CAR_PADDR + 0x050);
pub const CLK_RST_CONTROLLER_CLK_OUT_ENB_Y:            u32 = (CAR_PADDR + 0x298);
pub const CLK_RST_CONTROLLER_RST_DEV_L_SET:            u32 = (CAR_PADDR + 0x300);
pub const CLK_RST_CONTROLLER_RST_DEV_L_CLR:            u32 = (CAR_PADDR + 0x304);
pub const CLK_RST_CONTROLLER_RST_DEV_W_SET:            u32 = (CAR_PADDR + 0x438);
pub const CLK_RST_CONTROLLER_RST_DEV_W_CLR:            u32 = (CAR_PADDR + 0x43C);
pub const CLK_RST_CONTROLLER_UTMIP_PLL_CFG0:           u32 = (CAR_PADDR + 0x480);
pub const CLK_RST_CONTROLLER_UTMIP_PLL_CFG1:           u32 = (CAR_PADDR + 0x484);
pub const CLK_RST_CONTROLLER_UTMIP_PLL_CFG2:           u32 = (CAR_PADDR + 0x488);
pub const CLK_RST_CONTROLLER_UTMIPLL_HW_PWRDN_CFG0:    u32 = (CAR_PADDR + 0x52C);
pub const CLK_RST_CONTROLLER_CLK_SOURCE_USB2_HSIC_TRK: u32 = (CAR_PADDR + 0x6CC);
pub const XUSB_PADCTL_RST: u32 = (bit!(14));
pub const CLK_ENB_USBD:    u32 = (bit!(22));

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

pub const CAR_INFO_USBD: CarDeviceInfo = CarDeviceInfo {
    rst_dev_offset: 0x004,
    clk_out_enb_offset: 0x010,
    clk_source_offset: 0,
    dev_bit: 22,
    clk_source: 0,
    clk_divisor: 0,
};

impl CarDeviceInfo
{
    pub fn enable(&self) {

        self.disable();
        
        let rst_dev_reg: u32 = (CAR_VADDR + self.rst_dev_offset);
        let clk_enb_reg: u32 = (CAR_VADDR + self.clk_out_enb_offset);
        let clk_src_reg: u32 = (CAR_VADDR + self.clk_source_offset);

        unsafe
        {
            if(self.clk_source_offset != 0)
            {
                pokeio32(clk_src_reg, (self.clk_source as u32) << 29 | (self.clk_divisor as u32));
            }
        
            pokeio32(clk_enb_reg, peekio32(clk_enb_reg) | bit!(self.dev_bit));
            pokeio32(rst_dev_reg, peekio32(rst_dev_reg) & !(bit!(self.dev_bit)));
        }
    }
    
    pub fn disable(&self) {
        let rst_dev_reg: u32 = (CAR_VADDR + self.rst_dev_offset);
        let clk_enb_reg: u32 = (CAR_VADDR + self.clk_out_enb_offset);

        pokeio32(rst_dev_reg, peekio32(rst_dev_reg) | bit!(self.dev_bit));
        pokeio32(clk_enb_reg, peekio32(clk_enb_reg) & !(bit!(self.dev_bit)));
    }
    
    pub fn isEnabled(&self) -> bool {
        let rst_dev_reg: u32 = (CAR_VADDR + self.rst_dev_offset);
        let clk_enb_reg: u32 = (CAR_VADDR + self.clk_out_enb_offset);

        unsafe
        {
            let rst_dev = peekio32(rst_dev_reg);
            let clk_enb = peekio32(clk_enb_reg);
            
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
