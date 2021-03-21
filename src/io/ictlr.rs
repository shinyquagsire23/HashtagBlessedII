/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![allow(warnings, unused)]

use crate::util::*;

pub const PRI_ICTLR_BASE:   u32 = 0x60004000;
pub const SEC_ICTLR_BASE:   u32 = 0x60004100;
pub const TRI_ICTLR_BASE:   u32 = 0x60004200;
pub const QUAD_ICTLR_BASE:  u32 = 0x60004300;
pub const PENTA_ICTLR_BASE: u32 = 0x60004400;
pub const HEXA_ICTLR_BASE:  u32 = 0x60004500;

pub const ICTLR_BITS:  i32 = (32);
pub const ICTLR_MIN:   i32 = (1);
pub const ICTLR_MAX:   i32 = (6);
pub const ICTLR_COUNT: i32 = (6);

pub const ICTLR_CPU0:  u32 = (0);
pub const ICTLR_CPU1:  u32 = (1);
pub const ICTLR_CPU2:  u32 = (2);
pub const ICTLR_CPU3:  u32 = (3);
pub const ICTLR_COP:   u32 = (4);

pub fn ictlr_base(idx: i32) -> u32
{
    match idx {
        1 => return PRI_ICTLR_BASE,
        2 => return SEC_ICTLR_BASE,
        3 => return TRI_ICTLR_BASE,
        4 => return QUAD_ICTLR_BASE,
        5 => return PENTA_ICTLR_BASE,
        6 => return HEXA_ICTLR_BASE,
        _ => return 0,
    }
}

pub struct ICTLRSet
{
    ICTLR_VIRQ_CPU: MMIOReg,
    ICTLR_VIRQ_COP: MMIOReg,
    ICTLR_VFIQ_CPU: MMIOReg,
    ICTLR_VFIQ_COP: MMIOReg,
    ICTLR_ISR: MMIOReg,
    ICTLR_FIR: MMIOReg,
    ICTLR_FIR_SET: MMIOReg,
    ICTLR_FIR_CLR: MMIOReg,
    ICTLR_CPU_IER: MMIOReg,
    ICTLR_CPU_IER_SET: MMIOReg,
    ICTLR_CPU_IER_CLR: MMIOReg,
    ICTLR_CPU_IEP_CLASS: MMIOReg,
    ICTLR_COP_IER: MMIOReg,
    ICTLR_COP_IER_SET: MMIOReg,
    ICTLR_COP_IER_CLR: MMIOReg,
    ICTLR_COP_IEP_CLASS: MMIOReg,

    ICTLR_VIRQ_CPU1: MMIOReg,
    ICTLR_VFIQ_CPU1: MMIOReg,
    ICTLR_CPU1_IER: MMIOReg,
    ICTLR_CPU1_IER_SET: MMIOReg,
    ICTLR_CPU1_IER_CLR: MMIOReg,
    ICTLR_CPU1_IEP_CLASS: MMIOReg,
    ICTLR_VIRQ_CPU2: MMIOReg,
    ICTLR_VFIQ_CPU2: MMIOReg,
    ICTLR_CPU2_IER: MMIOReg,
    ICTLR_CPU2_IER_SET: MMIOReg,
    ICTLR_CPU2_IER_CLR: MMIOReg,
    ICTLR_CPU2_IEP_CLASS: MMIOReg,
    ICTLR_VIRQ_CPU3: MMIOReg,
    ICTLR_VFIQ_CPU3: MMIOReg,
    ICTLR_CPU3_IER: MMIOReg,
    ICTLR_CPU3_IER_SET: MMIOReg,
    ICTLR_CPU3_IER_CLR: MMIOReg,
    ICTLR_CPU3_IEP_CLASS: MMIOReg,
}

impl ICTLRSet
{
    pub fn new(set: i32) -> Self
    {
        let baseaddr: u32 = ictlr_base(set);
        let mut retval: ICTLRSet = ICTLRSet {
            ICTLR_VIRQ_CPU: MMIOReg::new(baseaddr + 0x00),
            ICTLR_VIRQ_COP: MMIOReg::new(baseaddr + 0x04),
            ICTLR_VFIQ_CPU: MMIOReg::new(baseaddr + 0x08),
            ICTLR_VFIQ_COP: MMIOReg::new(baseaddr + 0x0C),
            ICTLR_ISR: MMIOReg::new(baseaddr + 0x10),
            ICTLR_FIR: MMIOReg::new(baseaddr + 0x14),
            ICTLR_FIR_SET: MMIOReg::new(baseaddr + 0x18),
            ICTLR_FIR_CLR: MMIOReg::new(baseaddr + 0x1C),
            ICTLR_CPU_IER: MMIOReg::new(baseaddr + 0x20),
            ICTLR_CPU_IER_SET: MMIOReg::new(baseaddr + 0x24),
            ICTLR_CPU_IER_CLR: MMIOReg::new(baseaddr + 0x28),
            ICTLR_CPU_IEP_CLASS: MMIOReg::new(baseaddr + 0x2C),
            ICTLR_COP_IER: MMIOReg::new(baseaddr + 0x30),
            ICTLR_COP_IER_SET: MMIOReg::new(baseaddr + 0x34),
            ICTLR_COP_IER_CLR: MMIOReg::new(baseaddr + 0x38),
            ICTLR_COP_IEP_CLASS: MMIOReg::new(baseaddr + 0x3C),

            ICTLR_VIRQ_CPU1: MMIOReg::new(baseaddr + 0x60),
            ICTLR_VFIQ_CPU1: MMIOReg::new(baseaddr + 0x64),
            ICTLR_CPU1_IER: MMIOReg::new(baseaddr + 0x68),
            ICTLR_CPU1_IER_SET: MMIOReg::new(baseaddr + 0x6C),
            ICTLR_CPU1_IER_CLR: MMIOReg::new(baseaddr + 0x70),
            ICTLR_CPU1_IEP_CLASS: MMIOReg::new(baseaddr + 0x74),
            ICTLR_VIRQ_CPU2: MMIOReg::new(baseaddr + 0x78),
            ICTLR_VFIQ_CPU2: MMIOReg::new(baseaddr + 0x7C),
            ICTLR_CPU2_IER: MMIOReg::new(baseaddr + 0x80),
            ICTLR_CPU2_IER_SET: MMIOReg::new(baseaddr + 0x84),
            ICTLR_CPU2_IER_CLR: MMIOReg::new(baseaddr + 0x88),
            ICTLR_CPU2_IEP_CLASS: MMIOReg::new(baseaddr + 0x8C),
            ICTLR_VIRQ_CPU3: MMIOReg::new(baseaddr + 0x90),
            ICTLR_VFIQ_CPU3: MMIOReg::new(baseaddr + 0x94),
            ICTLR_CPU3_IER: MMIOReg::new(baseaddr + 0x98),
            ICTLR_CPU3_IER_SET: MMIOReg::new(baseaddr + 0x9C),
            ICTLR_CPU3_IER_CLR: MMIOReg::new(baseaddr + 0xA0),
            ICTLR_CPU3_IEP_CLASS: MMIOReg::new(baseaddr + 0xA4),
        };
        
        return retval;
    }
    
    pub fn irq_en(&mut self, bit: i32)
    {
        self.ICTLR_CPU_IEP_CLASS &= !bit!(bit);
        self.ICTLR_CPU_IER_SET.w32(bit!(bit));

        self.ICTLR_CPU1_IEP_CLASS &= !bit!(bit);
        self.ICTLR_CPU1_IER_SET &= !bit!(bit);
        self.ICTLR_CPU2_IEP_CLASS &= !bit!(bit);
        self.ICTLR_CPU2_IER_SET &= !bit!(bit);
        self.ICTLR_CPU3_IEP_CLASS &= !bit!(bit);
        self.ICTLR_CPU3_IER_SET &= !bit!(bit);
    }
    
    pub fn irq_ack(&mut self, bit: i32)
    {
        self.ICTLR_FIR_CLR.w32(bit!(bit));
    }
}
