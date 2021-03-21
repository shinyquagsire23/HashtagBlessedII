/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![no_std]
#![no_main]
#![feature(llvm_asm)]
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

use util::*;
use io::uart::*;
use io::uart::UARTDevicePort::*;
use core::panic::PanicInfo;
use io::timer::*;
use io::smmu::*;
use arm::fpu::*;
use arm::gic::*;
use arm::virtualization::*;
use arm::cache::*;
use arm::mmu::*;
use vm::virq::*;
use vm::vmmio::*;
use vm::vsvc::*;
use vm::vmmu::*;
use vm::funcs::*;
use usbd::usbd::*;
use usbd::cdc::*;
use logger::*;
use alloc::vec::Vec;
use hos::kernel::KERNEL_START;

global_asm!(include_str!("start.s"));

const KERN_DATA: &[u8] = include_bytes!("../data/0_kernel_80060000.bin");

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

    vmmio_init();
    vsvc_init();

    usbd_recover();
    gic.enable_interrupt(IRQ_T210_USB, 0);
    tegra_irq_en(IRQNUM_T210_USB as i32);
    
    println!("");
    println!("");
    println!("");
    println!("Waddup from EL2!");
    
    while (!cdc_active()){timer_wait(1);}
    cdc_enable();
    
    println!("USB connection recovered!");

    timer_trap_el1();
    
    println!("Begin copy to {:16x}... {:x}\n\r", ipaddr_to_paddr(KERNEL_START), peek32(to_u64ptr!(&KERN_DATA[0])));
    memcpy32(ipaddr_to_paddr(KERNEL_START), to_u64ptr!(&KERN_DATA[0]), KERN_DATA.len());

    // Set up SVC pre/post hooks
    let daifclr_2_instr: u32 = 0xd50342ff;
    let search_start = ipaddr_to_paddr(KERNEL_START);
    let search_end = search_start + KERN_DATA.len() as u64;
    let mut a64_hooked = false;
    let mut a32_hooked = false;
    let mut search = search_start;
    loop
    {
        // SVC handlers are identifiable by
        // msr DAIFClr, #0x2
        // blr x19 / blr x11 for A32/A64
        // msr DAIFSet, #0x2
        if (peek32(search) == daifclr_2_instr && peek16(search + 6) == 0xd63f)
        {
            println!("Hooking addr {:16x}\n\r", search);
            if (peek32(search + 4) == 0xd63f0160) // A64
            {
                poke32(search + 0, 0xd4000002 | (1 << 5)); // HVC #1 instruction
                poke32(search + 8, 0xd4000002 | (2 << 5)); // HVC #2 instruction
                a64_hooked = true;
            }
            else if (peek32(search + 4) == 0xd63f0260) // A32
            {
                poke32(search + 0, 0xd4000002 | (3 << 5)); // HVC #3 instruction
                poke32(search + 8, 0xd4000002 | (4 << 5)); // HVC #4 instruction
                a32_hooked = true;
            }
        }

        if (a64_hooked && a32_hooked) { break; }
        
        search += 4;
    }

    // Finalize things
    dcache_flush(ipaddr_to_paddr(KERNEL_START), 0x10000000);
    icache_invalidate(ipaddr_to_paddr(KERNEL_START), 0x10000000);
    println!("Done copy...");
    vttbr_construct();
    
    println!("translate {:16x} -> {:16x}", KERNEL_START, translate_el1_stage12(KERNEL_START));
    
    println!("Dropping to EL1");
    unsafe
    {
        drop_to_el1(KERNEL_START);
        loop {}
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
