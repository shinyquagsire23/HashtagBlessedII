/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![no_std]
#![no_main]
#![feature(global_asm)]
#![allow(unused_parens)]
#![allow(unused)]
#![allow(non_snake_case)]

#[macro_use] mod util;

#[macro_use]
extern crate lazy_static;

mod io;
mod hos;
mod arm;
mod usbd;
mod vm;
mod modules;

use util::t210_reset;
use io::uart::*;
use io::uart::UARTDevicePort::*;
use core::panic::PanicInfo;
use io::timer::*;
use arm::fpu::*;

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main_warm() 
{
    let mut uart_a: UARTDevice = UARTDevice::new(UartA, 115200);
    
    uart_a.writeStr("Yo from EL2\n\r")
}

#[no_mangle]
pub extern "C" fn main_cold() 
{
    fpuEnable();
    
    let mut uart_a: UARTDevice = UARTDevice::new(UartA, 115200);
    
    uart_a.writeStr("\n\r\n\r\n\rWaddup from EL2!\n\r");
    uart_a.waitForWrite();
    timerWait(1000000);
}

#[no_mangle]
pub extern "C" fn exception_handle() 
{
    
}

#[no_mangle]
pub extern "C" fn irq_handle() 
{
    
}

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}
