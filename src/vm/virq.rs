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
use crate::usbd::usbd::*;
use crate::usbd::cdc::*;

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
pub const LR_INVALID_SLOT: u32 = (0xFF);
pub const LR_HWINT:        u32 = (bit!(31));
pub const LR_STS_SHIFT:    u32 = (28);
pub const LR_PRIO_SHIFT:   u32 = (23);
pub const LR_PRIO_MASK:    u32 = (0x1F);
pub const LR_IE_EOI:       u32 = (bit!(19));
pub const LR_SHIFT_VCPU:   u32 = (10);
pub const LR_SHIFT_PIRQ:   u32 = (10);
pub const LR_SHIFT_VIRQ:   u32 = (0);

pub const LR_STS_PENDING: u32 = (1);

pub const GICH_INT_NP:  u32 = (bit!(3)); // no pending LRs
pub const GICH_INT_U:   u32 = (bit!(1)); // underflow
pub const GICH_INT_EOI: u32 = (bit!(0)); // end of interrupt IRQ

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

pub fn virq_handle() -> u64
{
    let mut gic: GIC = GIC::new();
    
    //let start_ticks = vsysreg_getticks();
    //let end_ticks = start_ticks;

    let rpr = gic.get_rpr();
    let vrpr = gic.get_vrpr();
    let vcpu = gic.get_int_vcpu();
    let int_id = gic.get_int_id();

    let iar = gic.get_iar();
    let iar_vcpu = gic.get_iar_vcpu();
    let iar_int_id = gic.get_iar_int_id();

    gic.enable_interrupt(IRQ_EL2_GIC_MAINTENANCE, get_core());
    gic.enable_interrupt(IRQ_EL2_TIMER, get_core());

    //TODO
    /*if (get_core() == 3)
    {
        last_core_ret = get_elr_el2();
        last_core_name = vsvc_get_curpid_name();
    }*/
    //    printf("(core %u) a\n\r", get_core());

    gic.enable_interrupt(IRQ_T210_USB, 0);
    tegra_irq_en(20);

    gic.set_gich_vmcr();

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

        gic.set_eoir(iar);
        gic.set_dir(iar);
        
        //TODO
        //end_ticks = vsysreg_getticks();
        //vsysreg_addoffset(end_ticks - start_ticks);
        return get_elr_el2();
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
               get_elr_el2());
    }*/

    // TODO was this correct...?
    if (iar != IRQ_INVALID as u32)
    {
        // software interrupts can be fully signalled
        if (tegra_irq_is_sgi(int_id))
        {
            gic.set_eoir(iar);
            gic.set_dir(iar);
        }
        else
        {
            // Hardware interrupts need to be acknowledged so that another IRQ
            // doesn't show up, however GICC_DIR will need to be written once it
            // completes
            gic.set_eoir(iar);
        }
    }

    //if (get_core() == 3)
     //   printf("(core %u) b\n\r", get_core());
    //TODO
    //end_ticks = vsysreg_getticks();
    //vsysreg_addoffset(end_ticks - start_ticks);
    return get_elr_el2();
}
