/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![allow(unused_parens)]
#![allow(unused)]
#![allow(non_snake_case)]
#![feature(unboxed_closures, fn_traits)]

#[macro_use] mod util;

#[macro_use]
extern crate lazy_static;

mod io;

#[macro_use] mod logger;

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
use io::smmu::*;
use arm::fpu::*;
use arm::gic::*;
use vm::virq::*;
use usbd::usbd::*;
use logger::*;

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main_warm() 
{
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    
    uart_a.writeStr("Yo from EL2\n\r");
    timerWait(1000000);
}

#[no_mangle]
pub extern "C" fn main_cold() 
{
    fpuEnable();
    
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    uart_a.init(115200);
    
    logger_init();
    smmu_init();
    
    let mut gic: GIC = GIC::new();
    gic.init();

    //vmmio_init();
    //vsvc_init();

    //gic.enableInterrupt(IRQ_T210_USB, 0);
    //tegra_irq_en(IRQNUM_T210_USB as i32);
    usbd_recover();
    
    log("\n\r\n\r\n\rWaddup from EL2!\n\r");
    timerWait(1000000);
    loop {}
}

#[no_mangle]
pub extern "C" fn exception_handle() 
{
    log("exception?\n\r");
    timerWait(1000000);
    loop {}
}

#[no_mangle]
pub extern "C" fn irq_handle() 
{
    log("IRQ?\n\r");
    timerWait(1000000);
    loop {}
}

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    log("panic?\n\r");
    timerWait(1000000);
    loop {}
}
