/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use crate::util::*;
use crate::arm::threading::*;

pub const GICD_BASE: u32 = 0x50041000;
pub const GICC_BASE: u32 = 0x50042000;
pub const GICH_BASE: u32 = 0x50044000; // processor-specific 0x5000? 0x5000, 0x5200, ...
pub const GICV_BASE: u32 = 0x50046000;

pub struct GICDRegs
{
    GICD_CTLR: MMIOReg,
    GICD_TYPER: MMIOReg,
    GICD_IIDR: MMIOReg,
    GICD_IGROUPR: MMIOReg,
    GICD_ISENABLER: MMIOReg,
    GICD_ICENABLER: MMIOReg,
    GICD_ISPENDR: MMIOReg,
    GICD_ICPENDR: MMIOReg,
    GICD_ICDABR: MMIOReg,
    GICD_ICACTIVER: MMIOReg,
    GICD_IPRIORITYR: MMIOReg,
    GICD_ITARGETSR: MMIOReg,
    GICD_ICFGR: MMIOReg,
    GICD_SGIR: MMIOReg,
    GICD_CPENDSGIR: MMIOReg,
    GICD_SPENDSGIR: MMIOReg,
}

pub struct GICCRegs
{
    GICC_CTLR: MMIOReg,
    GICC_PMR: MMIOReg,
    GICC_BPR: MMIOReg,
    GICC_IAR: MMIOReg,
    GICC_EOIR: MMIOReg,
    GICC_RPR: MMIOReg,
    GICC_HPPIR: MMIOReg,
    GICC_APR: MMIOReg,
    GICC_NSAPR: MMIOReg,
    GICC_IIDR: MMIOReg,
    GICC_DIR: MMIOReg
}

pub struct GICHRegs
{
    GICH_HCR: MMIOReg,
    GICH_VTR: MMIOReg,
    GICH_VMCR: MMIOReg,
    GICH_MISR: MMIOReg,
    GICH_EISR0: MMIOReg,
    GICH_EISR1: MMIOReg,
    GICH_ELSR0: MMIOReg,
    GICH_ELSR1: MMIOReg,
    GICH_APR: MMIOReg,
    GICH_LR: MMIOReg, // 0x100 ... 0x1FC
}

pub struct GICVRegs
{
    GICV_CTLR: MMIOReg,
    GICV_PMR: MMIOReg,
    GICV_BPR: MMIOReg,
    GICV_IAR: MMIOReg,
    GICV_EOIR: MMIOReg,
    GICV_RPR: MMIOReg,
    GICV_HPPIR: MMIOReg,
    GICV_APR: MMIOReg,
    GICV_NSAPR: MMIOReg,
    GICV_IIDR: MMIOReg,
    GICV_DIR: MMIOReg,
}

impl GICDRegs
{
    pub fn new() -> Self {
        let mut retval: GICDRegs = GICDRegs {
            GICD_CTLR:     MMIOReg::new(GICD_BASE + 0x0000),
            GICD_TYPER:    MMIOReg::new(GICD_BASE + 0x0004),
            GICD_IIDR:     MMIOReg::new(GICD_BASE + 0x0008),
            
            // vu32*
            GICD_IGROUPR:   MMIOReg::new(GICD_BASE + 0x0080),
            GICD_ISENABLER: MMIOReg::new(GICD_BASE + 0x0100),
            GICD_ICENABLER: MMIOReg::new(GICD_BASE + 0x0180),
            GICD_ISPENDR:   MMIOReg::new(GICD_BASE + 0x0200),
            GICD_ICPENDR:   MMIOReg::new(GICD_BASE + 0x0280),
            GICD_ICDABR:    MMIOReg::new(GICD_BASE + 0x0300),
            GICD_ICACTIVER: MMIOReg::new(GICD_BASE + 0x0280),
            
            // vu8*
            GICD_IPRIORITYR: MMIOReg::new(GICD_BASE + 0x0400),
            GICD_ITARGETSR: MMIOReg::new(GICD_BASE + 0x0800),
            
            // vu32*
            GICD_ICFGR: MMIOReg::new(GICD_BASE + 0x0C00),
            
            // vu32
            GICD_SGIR: MMIOReg::new(GICD_BASE + 0x0F00),
            
            // vu8*
            GICD_CPENDSGIR: MMIOReg::new(GICD_BASE + 0x0F20),
            GICD_SPENDSGIR: MMIOReg::new(GICD_BASE + 0x0F20)
        };
        
        return retval;
    }
    
    pub fn enableInterrupt(&mut self, num: u16, core: u8)
    {
        let reg: u16 = num / 32;
        let bit: u16 = num % 32;
        
        let mut icpendr_reg: MMIOReg = self.GICD_ICPENDR.idx32(reg as u32);
        let mut isenabler_reg: MMIOReg = self.GICD_ISENABLER.idx32(reg as u32);
        let mut itargetsr_reg: MMIOReg = self.GICD_ITARGETSR.idx8(num as u32);
        let mut ipriorityr_reg: MMIOReg = self.GICD_IPRIORITYR.idx8(num as u32);
    
        // Enable the interrupt
        icpendr_reg |= bit!(bit);
        isenabler_reg |= bit!(bit);
        
        // Route to CPUn
        itargetsr_reg.set8(bit!(core));
        ipriorityr_reg.w8(0x7F);
    }
    
    pub fn disableInterrupt(&mut self, num: u16, core: u8)
    {
        let reg: u16 = num / 32;
        let bit: u16 = num % 32;
        
        let mut icpendr_reg: MMIOReg = self.GICD_ICPENDR.idx32(reg as u32);
        let mut isenabler_reg: MMIOReg = self.GICD_ISENABLER.idx32(reg as u32);
        let mut itargetsr_reg: MMIOReg = self.GICD_ITARGETSR.idx8(num as u32);
        let mut ipriorityr_reg: MMIOReg = self.GICD_IPRIORITYR.idx8(num as u32);
    
        // Enable the interrupt
        icpendr_reg |= bit!(bit);
        isenabler_reg |= bit!(bit);
        
        // Route to CPUn
        itargetsr_reg.unset8(bit!(core));
        ipriorityr_reg.w8(0xFF);
    }
    
    pub fn clearInterrupt(&mut self, num: u16, core: u8)
    {
        let reg: u16 = num / 32;
        let bit: u16 = num % 32;
        
        let mut icpendr_reg: MMIOReg = self.GICD_ICPENDR.idx32(reg as u32);

        // Enable the interrupt
        icpendr_reg |= bit!(bit);
    }
    
    pub fn disableDistribution(&mut self)
    {
        let core: u8 = getCore();
        if (core == 0)
        {
            self.GICD_CTLR.w32(0);
        }
    }
    
    pub fn enableDistribution(&mut self)
    {
        let core: u8 = getCore();
        if (core == 0)
        {
            self.GICD_CTLR.w32(1)
        }
    }
    
    pub fn init(&mut self)
    {
        for i in 0..7
        {
            let icenabler: MMIOReg = self.GICD_ICENABLER.idx32(i);
            let icpendr: MMIOReg = self.GICD_ICPENDR.idx32(i);
            let icactiver: MMIOReg = self.GICD_ICACTIVER.idx32(i);
            let igroupr: MMIOReg = self.GICD_IGROUPR.idx32(i);

            icenabler.w32(0xFFFFFFFF);
            icpendr.w32(0xFFFFFFFF);
            icactiver.w32(0xFFFFFFFF);
            igroupr.w32(0x0);
        }
        
        for i in 0..32
        {
            let mut itargetsr_reg: MMIOReg = self.GICD_ITARGETSR.idx8(i);
            let mut ipriorityr_reg: MMIOReg = self.GICD_IPRIORITYR.idx8(i);
        
            itargetsr_reg.w8(1);
            ipriorityr_reg.w8(0x7F);
        }
        
        for i in 0..1
        {
            let icfgr: MMIOReg = self.GICD_ICFGR.idx32(i);
            icfgr.w32(0);
        }
    }
}

impl GICCRegs
{
    pub fn new() -> Self {
        let mut retval: GICCRegs = GICCRegs {
            GICC_CTLR:  MMIOReg::new(GICC_BASE + 0x0000),
            GICC_PMR:   MMIOReg::new(GICC_BASE + 0x0004),
            GICC_BPR:   MMIOReg::new(GICC_BASE + 0x0008),
            GICC_IAR:   MMIOReg::new(GICC_BASE + 0x000C),
            GICC_EOIR:  MMIOReg::new(GICC_BASE + 0x0010),
            GICC_RPR:   MMIOReg::new(GICC_BASE + 0x0014),
            GICC_HPPIR: MMIOReg::new(GICC_BASE + 0x0018),
            
            // vu32*
            GICC_APR:   MMIOReg::new(GICC_BASE + 0x00D0),
            GICC_NSAPR: MMIOReg::new(GICC_BASE + 0x00E0),
            
            // vu32
            GICC_IIDR:  MMIOReg::new(GICC_BASE + 0x00FC),
            GICC_DIR:   MMIOReg::new(GICC_BASE + 0x1000),
        };
        
        return retval;
    }
    
    pub fn enableEIO(&mut self)
    {
        self.GICC_CTLR |= (bit!(0) | bit!(9));
    }
    
    pub fn enableSignaling(&mut self)
    {
        self.GICC_CTLR.w32(0x1);
    }
    
    pub fn disableSignaling(&mut self)
    {
        self.GICC_CTLR.w32(0x0);
    }
    
    pub fn maskAll(&mut self)
    {
        self.GICC_PMR.w32(0xFF);
    }
    
    pub fn unmaskAll(&mut self)
    {
        self.GICC_PMR.w32(0);
    }
    
    pub fn setBPR(&mut self, val: u32)
    {
        self.GICC_BPR.w32(val);
    }
}

impl GICHRegs
{
    pub fn new() -> Self {
        let mut retval: GICHRegs = GICHRegs {
            GICH_HCR:   MMIOReg::new(GICH_BASE + 0x0000),
            GICH_VTR:   MMIOReg::new(GICH_BASE + 0x0004),
            GICH_VMCR:  MMIOReg::new(GICH_BASE + 0x0008),
            GICH_MISR:  MMIOReg::new(GICH_BASE + 0x0010),
            GICH_EISR0: MMIOReg::new(GICH_BASE + 0x0020),
            GICH_EISR1: MMIOReg::new(GICH_BASE + 0x0024),
            GICH_ELSR0: MMIOReg::new(GICH_BASE + 0x0030),
            GICH_ELSR1: MMIOReg::new(GICH_BASE + 0x0034),
            GICH_APR:   MMIOReg::new(GICH_BASE + 0x00F0),
            
            // vu32*
            GICH_LR:    MMIOReg::new(GICH_BASE + 0x0100), // 0x100 ... 0x1FC
        };
        
        return retval;
    }
    
    pub fn init(&mut self)
    {
        self.GICH_HCR |= bit!(0);
        self.GICH_VMCR |= bit!(0);// | BIT(9);// | BIT(9);// | BIT(9);
    }
}

impl GICVRegs
{
    pub fn new() -> Self {
        let mut retval: GICVRegs = GICVRegs {
            GICV_CTLR:  MMIOReg::new(GICV_BASE + 0x0000),
            GICV_PMR:   MMIOReg::new(GICV_BASE + 0x0004),
            GICV_BPR:   MMIOReg::new(GICV_BASE + 0x0008),
            GICV_IAR:   MMIOReg::new(GICV_BASE + 0x000C),
            GICV_EOIR:  MMIOReg::new(GICV_BASE + 0x0010),
            GICV_RPR:   MMIOReg::new(GICV_BASE + 0x0014),
            GICV_HPPIR: MMIOReg::new(GICV_BASE + 0x0018),
            GICV_APR:   MMIOReg::new(GICV_BASE + 0x00D0),
            GICV_NSAPR: MMIOReg::new(GICV_BASE + 0x00E0),
            GICV_IIDR:  MMIOReg::new(GICV_BASE + 0x00FC),
            GICV_DIR:   MMIOReg::new(GICV_BASE + 0x1000),
        };
        
        return retval;
    }
}

pub struct GIC
{
    gicd: GICDRegs,
    gicc: GICCRegs,
    gich: GICHRegs,
    gicv: GICVRegs,
}

impl GIC
{
    pub fn new() -> Self {
        let mut retval: GIC = GIC {
            gicd: GICDRegs::new(),
            gicc: GICCRegs::new(),
            gich: GICHRegs::new(),
            gicv: GICVRegs::new(),
        };
        
        return retval;
    }
    
    pub fn init(&mut self)
    {
        self.gicc.disableSignaling();
        self.gicd.disableDistribution();
        self.gicc.unmaskAll();
        self.gicc.setBPR(7);
        
        self.gicd.init();
        
        self.gicd.enableDistribution();
        self.gicc.enableSignaling();
        self.gicc.maskAll();

        // enable interrupts
        
        unsafe
        {
            asm!("msr daifclr, #0xF");
            asm!("isb");
        }
        
        //GICD_SGIR = 6;
        //GICD_SGIR = 6;
        //GICD_SGIR = 6;
        
        self.gich.init();
        self.gicc.enableEIO();
    }
    
    pub fn enableInterrupt(&mut self, num: u16, core: u8)
    {
        self.gicd.enableInterrupt(num, core);
    }
}
