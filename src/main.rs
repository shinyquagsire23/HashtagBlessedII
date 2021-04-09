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
#![feature(const_btree_new)]

#[macro_use]
extern crate alloc;
extern crate spin;

extern crate derive_more;

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
use crate::vm::vsmc::vsmc_get_warm_entrypoint;
use modules::ipc::ipc_init;

global_asm!(include_str!("start.s"));

const KERN_DATA: &[u8] = include_bytes!("../data/0_kernel_80060000.bin");

#[global_allocator]
static ALLOCATOR: HtbHeap = HtbHeap::empty();

#[repr(align(0x1000))]
struct PageAlignedHeapAlloc([u8; 0x400000]);

static mut HEAP_RES: PageAlignedHeapAlloc = PageAlignedHeapAlloc([0; 0x400000]);

#[no_mangle]
pub extern "C" fn main_warm(arg: u64) 
{
    // Warm boot from sleep mode
    if get_core() == 0
    {
        let mut uart_a: UARTDevice = UARTDevice::new(UartA);
        uart_a.init(115200);
    
        fpu_enable();
        smmu_init();
        vttbr_init();
        
        task_clear_all();
        logger_init();
        
        // Start IRQs for USB and tasking
        let mut gic: GIC = GIC::new();
        gic.init();
        irq_timer_init(&mut gic);

        // Start USB for real
        usbd_recover();
        gic.enable_interrupt(IRQ_T210_USB, 0);
        tegra_irq_en(IRQNUM_T210_USB as i32);
        
        task_run(blink_task());
        timer_wait(2000000);
    }
    
    
    println!("Hello from core {}! {:016x}", get_core(), vsysreg_getticks());

    // Set up new core with guest memory map
    let lock = critical_start();
    vttbr_transfer_newcore();
    //hcr_trap_wfe();
    //hcr_trap_wfi();
    critical_end(lock);
    
    // Set up new core with periodic timer
    let mut gic: GIC = GIC::new();
    gic.init();
    irq_timer_init(&mut gic);

    // Trap core's timer registers
    timer_trap_el1_access();
    
    let warm_entrypoint = vsmc_get_warm_entrypoint();

    println!("translate {:016x} -> {:016x}", warm_entrypoint, translate_el1_stage12(warm_entrypoint));
    unsafe { drop_to_el1(warm_entrypoint, arg); }
    
    loop{}
}

pub fn irq_timer_init(gic: &mut GIC)
{
    unsafe
    {
        // Quickly jump to IRQ handler where it sets its usual interval
        sysreg_write!("cnthp_tval_el2", 0x10);
        sysreg_write!("cnthp_ctl_el2", 1);
        gic.enable_interrupt(IRQ_EL2_TIMER, get_core());
    }
}

#[no_mangle]
pub extern "C" fn main_cold() 
{
    fpu_enable();
    
    unsafe { ALLOCATOR.init((HEAP_RES.0.as_ptr() as *const u8) as usize, 0x400000); }
    
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    uart_a.init(115200);
    
    // Initialize tasking for logger
    task_init();
    
    // Initialize logger
    logger_init();
    
    // Set up USB lockout weirdness w/ SMMU
    smmu_init();
    
    // Set up guest memory allocations
    vttbr_init();
    
    // Set up guest vMMIO, vSVC allocations
    vmmio_init();
    vsvc_init();
    ipc_init();
    
    // Start IRQs for USB and tasking
    let mut gic: GIC = GIC::new();
    gic.init();
    irq_timer_init(&mut gic);

    // Start USB for real
    usbd_recover();
    gic.enable_interrupt(IRQ_T210_USB, 0);
    tegra_irq_en(IRQNUM_T210_USB as i32);
    
    // Wait for host PC to enumerate device
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
    
    // Wait forever for debugger to attach
    while (!debug_active() && !debug_acked())
    {
        timer_wait(1000);
    }
    debug_enable();
    
    println!("USB connection recovered!");

    // Make sure debugger stayed attached
    timer_wait(2000000);
    while (!debug_active() && !debug_acked())
    {
        timer_wait(1000);
    }
    
    // Let debugger know we're booting
    log_cmd(&[1, 1, 0]);

    // Trap timer register accesses
    timer_trap_el1();

    // Start some tasks
    task_run(example_task());
    task_run(blink_task());
    
    //
    // Patching and hooking time...
    //
    println!("Begin copy to {:016x}... {:x}", ipaddr_to_paddr(KERNEL_START), peek32(to_u64ptr!(&KERN_DATA[0])));
    memcpy32(ipaddr_to_paddr(KERNEL_START), to_u64ptr!(&KERN_DATA[0]), KERN_DATA.len());
    
    log_cmd(&[1, 1, 1]);

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
    
    poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x280, 0xd4000002); // EL1 IRQ
    poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x280 - 4, 0xd69f03e0); // EL1 IRQ ERET
    poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x480, 0xd4000002); // EL0 IRQ
    poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x480 - 4, 0xd69f03e0); // EL0 IRQ ERET
    
    //poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x584, 0xd4000002); // lowerel serror
    //poke32(ipaddr_to_paddr(KERNEL_START) + 0x800 + 0x784, 0xd4000002); // lowerel serror
    
    poke32(ipaddr_to_paddr(KERNEL_START) + 0x4bcc0, 0xd4000002 | (6 << 5)); // el0 dabt/iabt

    // Finalize things
    dcache_flush(ipaddr_to_paddr(KERNEL_START), 0x10000000);
    icache_invalidate(ipaddr_to_paddr(KERNEL_START), 0x10000000);
    println!("Done copy...");

    // Set up guest memory
    let lock = critical_start();
    vttbr_construct();
    
    //hcr_trap_wfe();
    //hcr_trap_wfi();
    
    critical_end(lock);
    
    println!("translate {:016x} -> {:016x}", KERNEL_START, translate_el1_stage12(KERNEL_START));
    println!("Dropping to EL1");
    
    unsafe
    {
        drop_to_el1(KERNEL_START, 0);
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
        let spin = ["|", "/", "-", "\\"];
        i += 1;
        let spin_idx = (i & 3);
        print!("{} > {:<80} last {}ns max {}ns  \r", spin[spin_idx], debug_get_cmd_buf(), get_tasking_time(), get_tasking_time_max());
        
        // Let debugger know we're on home screen
        if vsvc_is_qlaunch_started() {
            log_cmd(&[1, 1, 0xFF]);
        }
        
        SleepNs::new(ms_to_ns(200)).await;
    }
}

#[no_mangle]
pub extern "C" fn exception_handle(which: i32, ctx: u64) -> u64
{
    unsafe
    {
        let mut ctx_slice: &'static mut [u64] = alloc::slice::from_raw_parts_mut(ctx as *mut u64, 0x40);
        return handle_exception(which, ctx_slice);
    }
}

#[no_mangle]
pub extern "C" fn irq_handle(which: i32, ctx: u64)  -> u64
{
    unsafe
    {
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

    timer_wait(4000000);
    unsafe { t210_reset(); }
    loop{}
}
