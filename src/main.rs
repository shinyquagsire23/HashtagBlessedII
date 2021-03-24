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
extern crate spin;

mod heap;

#[macro_use] mod util;

#[macro_use]
extern crate lazy_static;

#[macro_use] mod logger;

mod io;

mod hos;
mod arm;
mod usbd;
mod vm;
mod modules;
mod task;
mod exception_handler;

use heap::HtbHeap;
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
use usbd::debug::*;
use logger::*;
use alloc::vec::Vec;
use hos::kernel::KERNEL_START;
use task::*;
use task::executor::*;
use task::sleep::*;
use arm::ticks::*;
use arm::threading::get_core;
use exception_handler::*;
use vm::vsysreg::*;

global_asm!(include_str!("start.s"));

const KERN_DATA: &[u8] = include_bytes!("../data/0_kernel_80060000.bin");

#[global_allocator]
static ALLOCATOR: HtbHeap = HtbHeap::empty();

#[repr(align(0x1000))]
struct PageAlignedHeapAlloc([u8; 0x100000]);

static HEAP_RES: PageAlignedHeapAlloc = PageAlignedHeapAlloc([0; 0x100000]);

#[no_mangle]
pub extern "C" fn main_warm() 
{
    // TODO is this needed...?
    //dcache_invalidate(0xD0000000, 0x3000000);
    println!("Hello from core {}! {:016x}", get_core(), vsysreg_getticks());

    vttbr_transfer_newcore();
    
    let mut gic: GIC = GIC::new();
    gic.init();
    irq_timer_init(&mut gic);

    timer_trap_el1_access();

    println!("translate {:016x} -> {:016x}", KERNEL_START, translate_el1_stage12(KERNEL_START));
    unsafe { drop_to_el1(KERNEL_START); }
    
    loop{}
}

pub fn irq_timer_init(gic: &mut GIC)
{
    unsafe
    {
        let mut tmp: u64 = 0;
        tmp = 0x10; // quickly jump to IRQ handler so it can reset a better value
        asm!("msr CNTHP_TVAL_EL2, {0}", in(reg) tmp);
        tmp = 0x1;
        asm!("msr CNTHP_CTL_EL2, {0}", in(reg) tmp);
        gic.enable_interrupt(26, get_core());
    }
}

#[no_mangle]
pub extern "C" fn main_cold() 
{
    fpu_enable();
    
    unsafe { ALLOCATOR.init((HEAP_RES.0.as_ptr() as *const u8) as usize, 0x100000); }
    
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    uart_a.init(115200);
    
    // Initialize tasking for logger
    task_init();
    
    logger_init();
    smmu_init();
    vttbr_init();
    
    let mut gic: GIC = GIC::new();
    gic.init();
    irq_timer_init(&mut gic);

    vmmio_init();
    vsvc_init();

    usbd_recover();
    gic.enable_interrupt(IRQ_T210_USB, 0);
    tegra_irq_en(IRQNUM_T210_USB as i32);
    
    let mut time_out = 3000;
    while (!usbd_is_enumerated() && time_out > 0)
    {
        timer_wait(1000);
        time_out -= 1;
    }
    println!("Enumeration complete!");
    
    println!("");
    println!("");
    println!("");
    println!("Waddup from EL2!");
    
    while (!debug_active() && !debug_acked())
    {
        timer_wait(1000);
    }
    debug_enable();
    
    println!("USB connection recovered!");
    timer_wait(2000000);
    while (!debug_active() && !debug_acked())
    {
        timer_wait(1000);
    }

    timer_trap_el1();

    task_run(example_task());
    task_run(blink_task());
    
    println!("Begin copy to {:016x}... {:x}", ipaddr_to_paddr(KERNEL_START), peek32(to_u64ptr!(&KERN_DATA[0])));
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
            println!("Hooking addr {:016x}", search);
            if (peek32(search + 4) == 0xd63f0160) // A64
            {
                //poke32(search + 0, 0xd4000002 | (1 << 5)); // HVC #1 instruction
                //poke32(search + 8, 0xd4000002 | (2 << 5)); // HVC #2 instruction
                a64_hooked = true;
            }
            else if (peek32(search + 4) == 0xd63f0260) // A32
            {
                //poke32(search + 0, 0xd4000002 | (3 << 5)); // HVC #3 instruction
                //poke32(search + 8, 0xd4000002 | (4 << 5)); // HVC #4 instruction
                a32_hooked = true;
            }
        }

        if (a64_hooked && a32_hooked) { break; }
        
        search += 4;
    }
    
    
    //poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x584, 0xd4000002); // lowerel serror
    //poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x784, 0xd4000002); // lowerel serror
    
    poke32(ipaddr_to_paddr(KERNEL_START) + 0x4bcc0, 0xd4000002); // el0 dabt/iabt

    // Finalize things
    dcache_flush(ipaddr_to_paddr(KERNEL_START), 0x10000000);
    icache_invalidate(ipaddr_to_paddr(KERNEL_START), 0x10000000);
    println!("Done copy...");
    vttbr_construct();
    
    println!("translate {:016x} -> {:016x}", KERNEL_START, translate_el1_stage12(KERNEL_START));
    println!("Dropping to EL1");
    
    unsafe
    {
        drop_to_el1(KERNEL_START);
        loop {}
    }
}

async fn async_number() -> u32 
{
    SleepNs::new(secs_to_ns(10)).await;
    42
}

async fn example_task()
{
    let number = async_number().await;
    println!("async task returned: {}", number);
}

async fn blink_task()
{
    let mut i = 0;
    loop
    {
        i += 1;
        print!(if (i & 1 == 0) { ".\r" } else { "*\r" });
        SleepNs::new(ms_to_ns(500)).await;
    }
}

#[no_mangle]
pub extern "C" fn exception_handle(which: i32, ctx: u64) -> u64
{
    unsafe
    {
    // TODO does this memleak?
    let mut ctx_slice: &'static mut [u64] = alloc::slice::from_raw_parts_mut(ctx as *mut u64, 0x40);
    return handle_exception(which, ctx_slice);
    }
}

#[no_mangle]
pub extern "C" fn irq_handle(which: i32, ctx: u64)  -> u64
{
    unsafe
    {
    // TODO does this memleak?
    let mut ctx_slice: &'static mut [u64] = alloc::slice::from_raw_parts_mut(ctx as *mut u64, 0x40);
    return virq_handle(ctx_slice);
    }
    
}

#[panic_handler]
fn on_panic(panic_info: &PanicInfo) -> ! {
    critical_start();

    // Force as much as we can out of the logger
    //logger_unsafe_override();
    //log_process();
    
    //println_unsafe!("panic?");
    println_uarta!("{}", panic_info);
    println_unsafe!("{}", panic_info);

    timer_wait(1000000);
    unsafe { t210_reset(); }
    loop{}
}
