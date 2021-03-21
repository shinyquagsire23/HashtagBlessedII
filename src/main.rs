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
#![feature(unboxed_closures, fn_traits)]
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]

#[macro_use]
extern crate alloc;

mod heap;

use heap::HtbHeap;

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
use usbd::cdc::*;
use logger::*;
use alloc::vec::Vec;

global_asm!(include_str!("start.s"));

#[global_allocator]
static ALLOCATOR: HtbHeap = HtbHeap::empty();
static HEAP_RES: [u8; 0x100000] = [0; 0x100000];

#[no_mangle]
pub extern "C" fn main_warm() 
{
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    
    println!("Yo from EL2");
    timer_wait(1000000);
}

#[no_mangle]
pub extern "C" fn main_cold() 
{
    fpu_enable();
    
    unsafe { ALLOCATOR.init((&HEAP_RES[0] as *const u8) as usize, 0x100000); }
    
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    uart_a.init(115200);
    
    logger_init();
    smmu_init();
    
    let mut gic: GIC = GIC::new();
    gic.init();

    //vmmio_init();
    //vsvc_init();
    
    println!("example {:.1} test {:x} words {}", 1, 2, 3);

    usbd_recover();
    gic.enable_interrupt(IRQ_T210_USB, 0);
    tegra_irq_en(IRQNUM_T210_USB as i32);
    
    println!("");
    println!("");
    println!("");
    println!("Waddup from EL2!");
    
    println!("Wait for CDC");
    while (!cdc_active()){timer_wait(1);}
    cdc_enable();
    
    println!("Done init!");

    loop {
        timer_wait(1000000);
        println!("beep");
    }
}

#[no_mangle]
pub extern "C" fn exception_handle() 
{
    println!("exception?");
    timer_wait(1000000);
    loop {}
}

#[no_mangle]
pub extern "C" fn irq_handle()  -> u64
{
    return virq_handle();
}

#[panic_handler]
fn on_panic(panic_info: &PanicInfo) -> ! {
    println!("panic?");
    println!("{}", panic_info);
    /*if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        println!(s);
    } else {
        println!("Couldn't get error info!");
    }
    if let Some(location) = panic_info.location() {
       println!("panic occurred in file '{}' at line {}", location.file(), location.line());
    } else {
        println!("panic occurred but can't get location information...");
    }*/
    timer_wait(1000000);
    loop {}
}
