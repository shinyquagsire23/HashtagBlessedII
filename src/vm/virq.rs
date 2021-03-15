/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::util::*;
use crate::io::ictlr::*;

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

//pub const IRQ_IS_SGI(irqnum) ((u16)(irqnum & IRQ_MAX) < IRQ_PPI_START)

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
