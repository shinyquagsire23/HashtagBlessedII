/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::util::*;
use crate::io::ictlr::*;
use crate::arm::gic::*;
use crate::arm::threading::*;
use crate::arm::exceptions::*;
use crate::arm::ticks::*;
use crate::usbd::usbd::*;
use crate::task::*;
use crate::logger::*;
use crate::vm::vsysreg::*;
use crate::vm::vsvc::*;
use crate::vm::funcs::*;
use crate::io::timer::*;

pub const IRQNUM_T210_USB: u16 = 20;

pub const IRQ_MAX:                 u16 = (0x3FF);
pub const IRQ_INVALID:             u16 = (0x3FF);
pub const IRQ_T210_USB:            u16 = (IRQ_SPI_START + IRQNUM_T210_USB);
pub const IRQ_EL1_TIMER:           u16 = (30);
pub const IRQ_EL2_TIMER:           u16 = (26);
pub const IRQ_EL2_GIC_MAINTENANCE: u16 = (25);

pub const IAR_IRQ_MASK: u32 = (0x3FF);
pub const LR_IRQ_MASK:  u32 = (0x3FF);

// Software generated interrupts, used for inter-core comms
pub const IRQ_SGI_START: u16 = (0);

// Private peripheral interrupts, per-core timers and similar
pub const IRQ_PPI_START: u16 = (16);

// Shared peripheral interrupts, these map to the Tegra indices
pub const IRQ_SPI_START: u16 = (32);

// List register bits
pub const LR_INVALID_SLOT: u8 = (0xFF);
pub const LR_HWINT:        u32 = (bit!(31));
pub const LR_STS_SHIFT:    u32 = (28);
pub const LR_PRIO_SHIFT:   u32 = (23);
pub const LR_PRIO_MASK:    u32 = (0x1F);
pub const LR_IE_EOI:       u32 = (bit!(19));
pub const LR_SHIFT_VCPU:   u32 = (10);
pub const LR_SHIFT_PIRQ:   u32 = (10);
pub const LR_SHIFT_VIRQ:   u32 = (0);

pub const LR_STS_INVALID: u32 = (0);
pub const LR_STS_PENDING: u32 = (1);
pub const LR_STS_ACTIVE:  u32 = (2);
pub const LR_STS_PENDING_AND_ACTIVE: u32 = (1);

pub const GICH_INT_NP:  u32 = (bit!(3)); // no pending LRs
pub const GICH_INT_U:   u32 = (bit!(1)); // underflow
pub const GICH_INT_EOI: u32 = (bit!(0)); // end of interrupt IRQ

static mut IRQ_HEARTBEAT_DOWNSCALE: [u32; 8] = [0; 8];
static mut LAST_TASKING: u64 = 0;
static mut LAST_TASKING_MAX: u64 = 0;
static mut IRQ_ENABLE_ONCE: bool = true;

pub fn get_tasking_time() -> u64
{
    unsafe { return LAST_TASKING; }
}

pub fn get_tasking_time_max() -> u64
{
    unsafe { return LAST_TASKING_MAX; }
}


pub fn tegra_irq_is_sgi(irqnum: u16) -> bool
{
    return ((irqnum & IRQ_MAX) < IRQ_PPI_START);
}

pub fn tegra_irq_en(id: i32)
{
    let set: i32 = (id / ICTLR_BITS) + ICTLR_MIN;
    let bit: i32 = id % ICTLR_BITS;

    if(set > ICTLR_MAX)
    {
        return;
    }
    
    let mut ictlr: ICTLRSet = ICTLRSet::new(set);
    ictlr.irq_en(bit);
}

pub fn tegra_irq_ack(id: i32)
{
    let set: i32 = (id / ICTLR_BITS) + ICTLR_MIN;
    let bit: i32 = id % ICTLR_BITS;

    if(set > ICTLR_MAX)
    {
        return;
    }


    let mut ictlr: ICTLRSet = ICTLRSet::new(set);
    ictlr.irq_ack(bit);
}

// END tegra IRQ

pub fn virq_handle_fake(ctx: &mut [u64]) -> u64
{
    let mut gic: GIC = GIC::new();

    let elr_el2 = ctx[33];
    
    let hppir = gic.gicc.gicc_hppir.r32();
    let vcpu = ((hppir >> 10) & 0x7) as u8;
    let int_id = (hppir & 0x3FF) as u16;
    
    if (get_core() == 2 && int_id < 16 && unsafe { IRQ_ENABLE_ONCE })
    {
        gic.enable_interrupt(IRQ_T210_USB, get_core());
        tegra_irq_en(20);
        gic.enable_interrupt(IRQ_EL2_TIMER, get_core());
        unsafe { IRQ_ENABLE_ONCE = false; }
    }

    let mut show_irqs = false;

    if (int_id == IRQ_EL2_TIMER) // timer
    {
        sysreg_write!("cnthp_ctl_el2", 3);
        
        let iar = gic.get_iar();
        gic.set_eoir(iar);
        gic.set_dir(iar);
        
        let start = get_ticks();
        //TODO better place this?
        if (get_core() == 2) {
            task_advance();
        }
        let end = get_ticks();
        let mut total_ticks = end - start;
        unsafe 
        { 
            LAST_TASKING = ticks_to_ns(total_ticks);
            if (LAST_TASKING > LAST_TASKING_MAX) {
                LAST_TASKING_MAX = LAST_TASKING;
            }
        }
        
        if total_ticks > 0x30000 {
            total_ticks = 0x3000;
        }

        sysreg_write!("cnthp_tval_el2", 0x30000 - total_ticks);
        sysreg_write!("cnthp_ctl_el2", 1);

        unsafe
        {
            IRQ_HEARTBEAT_DOWNSCALE[get_core() as usize] += 1;
            if (IRQ_HEARTBEAT_DOWNSCALE[get_core() as usize] >= 0x10)
            {
                //println!("(core {}) heartbeat {:x} `{}`", get_core(), vsvc_get_curpid(), vsvc_get_curpid_name());
                IRQ_HEARTBEAT_DOWNSCALE[get_core() as usize] = 0;
            }
        }

        return elr_el2-8;
    }
    else if (int_id == IRQ_T210_USB)
    {
        let iar = gic.get_iar();
        gic.set_eoir(iar);
        gic.set_dir(iar);
        
        if (get_core() == 2) {
            irq_usb();
        }
        
        return elr_el2-8;
    }
    
    return elr_el2;
}

pub fn virq_handle(ctx: &mut [u64]) -> u64
{
    let mut gic: GIC = GIC::new();
    
    let start_ticks = vsysreg_getticks();
    let mut end_ticks = start_ticks;

    let elr_el2 = ctx[33];
    
    let hppir = gic.gicc.gicc_hppir.r32();
    let rpr = gic.get_rpr();
    let vrpr = gic.get_vrpr();
    let vcpu = ((hppir >> 10) & 0x7) as u8;//gic.get_int_vcpu();
    let int_id = (hppir & 0x3FF) as u16;//gic.get_int_id();

    
    //let iar_vcpu = ((iar >> 10) & 0x7) as u8;
    //let iar_int_id = (iar & 0x3FF) as u16;

    //gic.enable_interrupt(IRQ_EL2_GIC_MAINTENANCE, get_core());
    gic.enable_interrupt(IRQ_EL2_TIMER, get_core());

    gic.enable_interrupt(IRQ_T210_USB, 0);
    tegra_irq_en(20);

    gic.set_gich_vmcr();

    let mut show_irqs = false;

    if (int_id == IRQ_EL2_TIMER) // timer
    {
        //TODO better place this?
        if (get_core() == 0) {
            task_advance();
        }
        
        
        let mut tmp: u64 = 0;

        sysreg_write!("cnthp_tval_el2", 0x10000);
        sysreg_write!("cnthp_ctl_el2", 1);

        unsafe
        {
            IRQ_HEARTBEAT_DOWNSCALE[get_core() as usize] += 1;
            if (IRQ_HEARTBEAT_DOWNSCALE[get_core() as usize] >= 0x100)
            {
                println!("(core {}) heartbeat {:x} `{}`", get_core(), vsvc_get_curpid(), vsvc_get_curpid_name());
                IRQ_HEARTBEAT_DOWNSCALE[get_core() as usize] = 0;
            }
        }
        let iar = gic.get_iar();
        gic.set_eoir(iar);
        gic.set_dir(iar);
        
        end_ticks = vsysreg_getticks();
        vsysreg_addoffset(end_ticks - start_ticks);

        return elr_el2;
    }
    else if (int_id == IRQ_EL2_GIC_MAINTENANCE)
    {
        //TODO
        //println!("(core {}) maintenance misr {:08x} hcr {:08x} vmcr {:08x} eisr0 {:08x} eisr1 {:08x} elsr0 {:08x} elsr1 {:08x} gicv_ctlr {:08x} gicc_ctrl {:08x}", get_core(), gic.get_gich_misr(), gic.gich.gich_hcr.r32(), gic.gich.gich_vmcr.r32(), gic.gich.gich_eisr0.r32(), gic.gich.gich_eisr1.r32(), gic.gich.gich_elsr0.r32(), gic.gich.gich_elsr1.r32(), gic.gicv.gicv_ctlr.r32(), gic.gicc.gicc_ctlr.r32());
        
        gic.do_maintenance();
        
        let iar = gic.get_iar();
        gic.set_eoir(iar);
        gic.set_dir(iar);
        
        end_ticks = vsysreg_getticks();
        vsysreg_addoffset(end_ticks - start_ticks);
        
        return elr_el2;
    }
    else if (int_id == IRQ_T210_USB)
    {
        irq_usb();

        tegra_irq_ack(int_id as i32);

        let iar = gic.get_iar();
        gic.set_eoir(iar);
        gic.set_dir(iar);
        
        end_ticks = vsysreg_getticks();
        vsysreg_addoffset(end_ticks - start_ticks);
        

        
        return elr_el2;
    }
/*    else if (int_id != 0x3FF)
    {
        if (int_id == 0x1e) {
            //show_irqs = true;
        }

        gic.send_interrupt(int_id, vcpu, rpr);
        gic.process_queue();
    }

    if (show_irqs) 
    {
        println!("IRQ core {} (misr {:x} rpr {:x} id {:x}, vcpu {}, IAR {:04x}->{:04x} LR[0] {:08x} ELSR0 {:08x} hppir {:04x}->{:04x}, vrpr {:02x}->{:02x}) ret {:016x}",
               get_core(),
               gic.get_gich_misr(), rpr, int_id, vcpu,
               iar, gic.gicv.gicv_hppir.r32(),
               gic.gich.gich_lr.r32(), gic.gich.gich_elsr0.r32(),
               hppir, gic.gicc.gicc_hppir.r32(),
               vrpr, gic.gicv.gicv_rpr.r32(),
               elr_el2);
    }

    // TODO was this correct...?
    if (int_id != IRQ_INVALID)
    {
        // software interrupts can be fully signalled
        if (tegra_irq_is_sgi(int_id))
        {
            gic.set_eoir(iar);
            //gic.set_dir(iar);
        }
        else
        {
            // Hardware interrupts need to be acknowledged so that another IRQ
            // doesn't show up, however GICC_DIR will need to be written once it
            // completes
            gic.set_eoir(iar);
        }
    }*/

    //if (get_core() == 3)
     //   printf("(core {}) b\n\r", get_core());
    //TODO
    end_ticks = vsysreg_getticks();
    vsysreg_addoffset(end_ticks - start_ticks);
    
    return elr_el2;
}

pub fn critical_start() -> u64
{
    let mut daif: u64 = 0;
    unsafe {
        asm!("mrs {0}, daif", out(reg) daif);
        asm!("msr daifset, #0x2");
    }
    
    return daif;
    
    //let mut gic: GIC = GIC::new();
    //gic.disable_distribution();
}

pub fn critical_end(lock: u64)
{
    unsafe {
        asm!("msr daif, {0}", in(reg) lock);
    }
    
    //let mut gic: GIC = GIC::new();
    //gic.enable_distribution();
}
