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
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]

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
use arm::threading::*;
use arm::exceptions::*;
use alloc::vec::Vec;

global_asm!(include_str!("start.s"));

#[global_allocator]
static ALLOCATOR: HtbHeap = HtbHeap::empty();
static HEAP_RES: [u8; 0x100000] = [0; 0x100000];

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
    
    unsafe { ALLOCATOR.init((&HEAP_RES[0] as *const u8) as usize, 0x100000); }
    
    let mut uart_a: UARTDevice = UARTDevice::new(UartA);
    uart_a.init(115200);
    
    logger_init();
    smmu_init();
    
    let mut gic: GIC = GIC::new();
    gic.init();

    //vmmio_init();
    //vsvc_init();

    usbd_recover();
    gic.enableInterrupt(IRQ_T210_USB, 0);
    tegra_irq_en(IRQNUM_T210_USB as i32);
    
    log("\n\r\n\r\n\rWaddup from EL2!\n\r");
    
    log("Wait for CDC\n\r");
    while (!cdc_active()){timerWait(1);}
    cdc_enable();
    
    logln("Done init!");

    loop {
        timerWait(1000000);
        logln("beep");
    }
}

#[no_mangle]
pub extern "C" fn exception_handle() 
{
    log("exception?\n\r");
    timerWait(1000000);
    loop {}
}

#[no_mangle]
pub extern "C" fn irq_handle()  -> u64
{
    let mut gic: GIC = GIC::new();
    
    //let start_ticks = vsysreg_getticks();
    //let end_ticks = start_ticks;

    let rpr = gic.getRPR();
    let vrpr = gic.getVRPR();
    let vcpu = gic.getIntVCPU();
    let int_id = gic.getIntId();

    let iar = gic.getIAR();
    let iar_vcpu = gic.getIARVCPU();
    let iar_int_id = gic.getIARIntId();

    gic.enableInterrupt(IRQ_EL2_GIC_MAINTENANCE, getCore());
    gic.enableInterrupt(IRQ_EL2_TIMER, getCore());

    //TODO
    /*if (getCore() == 3)
    {
        last_core_ret = getELREL2();
        last_core_name = vsvc_get_curpid_name();
    }*/
    //    printf("(core %u) a\n\r", get_core());

    gic.enableInterrupt(IRQ_T210_USB, 0);
    tegra_irq_en(20);

    gic.set_GICH_VMCR();

    //TODO
    /*if (int_id == 0x1e)
    {
        lockup[get_core()]++;
    }*/

    if (int_id == 26) // timer
    {
        //TODO
        /*u32 tmp = 0;

        tmp = 0x80000;
        asm volatile ("msr CNTHP_TVAL_EL2, %0" : "=r" (tmp));
        tmp = 0x1;
        asm volatile ("msr CNTHP_CTL_EL2, %0" : "=r" (tmp));

        GICC_EOIR = iar;
        GICC_DIR = iar;

        if (++irq_heartbeat_downscale[get_core()] >= 0x100)
        {
            printf("(core %u) heartbeat %x %x %s %016llx %s\n\r", get_core(), lockup[get_core()], vsvc_get_curpid(), vsvc_get_curpid_name(), last_core_ret, last_core_name);
            irq_heartbeat_downscale[get_core()] = 0;

            if (get_core() == 3 && lockup[get_core()] == last_lockup[get_core()])
            {
                asm volatile ("mrs %0, CNTP_CVAL_EL0" : "=r" (tmp));
                printf("lockup %x\n\r", tmp);
                /*tmp = 100;
                asm volatile ("msr CNTP_TVAL_EL0, %0" : "=r" (tmp));
                tmp = 0x1;
                asm volatile ("msr CNTP_CTL_EL0, %0" : "=r" (tmp));
                gic_enable_interrupt(0x1e, get_core());
                GICC_EOIR = 0x1e;
                GICC_DIR = 0x1e;
                GICH_APR &= ~BIT(1);*/

            }
            last_lockup[get_core()] = lockup[get_core()];
        }
        else
        {
            cdc_send(NULL, 0);
        }



        goto _done;*/
    }
    else if (int_id == IRQ_EL2_GIC_MAINTENANCE)
    {
        //TODO
        //printf("(core %u) maintenance misr %08x hcr %08x vmcr %08x eisr0 %08x eisr1 %08x elsr0 %08x elsr1 %08x gicv_ctlr %08x gicc_ctrl %08x\n\r", get_core(), GICH_MISR, GICH_HCR, GICH_VMCR, GICH_EISR0, GICH_EISR1, GICH_ELSR0, GICH_ELSR1, GICV_CTLR, GICC_CTLR);
        /*if (GICH_MISR & GICH_INT_NP)
        {
            GICH_HCR &= ~GICH_INT_NP;
            //printf("a %x %08x %08x\n\r", GICH_EISR0, GICH_LR[0], GICH_LR[1]);
            //GICH_HCR |= GICH_INT_U;
        }
        else if (GICH_MISR & GICH_INT_U)
        {
            //printf("b %x %08x %08x\n\r", GICH_EISR0, GICH_LR[0], GICH_LR[1]);

            GICH_HCR &= ~GICH_INT_U;
        }
        else if (GICH_MISR & GICH_INT_EOI) // EOI
        {

        }

        process_queue();

        GICC_EOIR = iar;
        GICC_DIR = iar;
        goto _done;*/
    }
    else if (int_id == IRQ_T210_USB)
    {
        //mutex_lock(&irq_usb_mutex);
        cdc_disable();
        irq_usb();
        cdc_enable();
        //mutex_unlock(&irq_usb_mutex);

        tegra_irq_ack(int_id as i32);

        gic.setEIOR(iar);
        gic.setDIR(iar);
        
        //TODO
        //end_ticks = vsysreg_getticks();
        //vsysreg_addoffset(end_ticks - start_ticks);
        return getELREL2();
    }
    else if (iar_int_id != 0x3FF)
    {
        //TODO
        //send_interrupt(iar_int_id, iar_vcpu, rpr);
        //process_queue();
    }

    /*if (show_irqs) {
        printf("IRQ core %u (misr %x rpr %x id %x, vcpu %u, IAR %04x->%04x LR[0] %08x ELSR0 %08x hppir %04x->%04x, vrpr %02x->%02x) ret %016llx\n\r",
               get_core(),
               GICH_MISR, rpr, int_id, vcpu,
               iar, GICV_HPPIR,
               GICH_LR[0], GICH_ELSR0,
               hppir, GICC_HPPIR,
               vrpr, GICV_RPR,
               getELREL2());
    }*/

    // TODO was this correct...?
    if (iar != IRQ_INVALID as u32)
    {
        // software interrupts can be fully signalled
        if (tegra_irq_is_sgi(int_id))
        {
            gic.setEIOR(iar);
            gic.setDIR(iar);
        }
        else
        {
            // Hardware interrupts need to be acknowledged so that another IRQ
            // doesn't show up, however GICC_DIR will need to be written once it
            // completes
            gic.setEIOR(iar);
        }
    }

    //if (get_core() == 3)
     //   printf("(core %u) b\n\r", get_core());
    //TODO
    //end_ticks = vsysreg_getticks();
    //vsysreg_addoffset(end_ticks - start_ticks);
    return getELREL2();
}

#[panic_handler]
fn on_panic(panic_info: &PanicInfo) -> ! {
    log("panic?\n\r");
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        log(s);
        log("\n\r");
    } else {
        log("Couldn't get error info!\n\r");
    }
    if let Some(location) = panic_info.location() {
       log("panic occurred in file '");
       log(location.file());
       log("' at line ");
       logu32(location.line());
       log("\n\r");
    } else {
        log("panic occurred but can't get location information...\n\r");
    }
    timerWait(1000000);
    loop {}
}
