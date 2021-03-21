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
    gicd_ctlr: MMIOReg,
    gicd_typer: MMIOReg,
    gicd_iidr: MMIOReg,
    gicd_igroupr: MMIOReg,
    gicd_isenabler: MMIOReg,
    gicd_icenabler: MMIOReg,
    gicd_ispendr: MMIOReg,
    gicd_icpendr: MMIOReg,
    gicd_icdabr: MMIOReg,
    gicd_icactiver: MMIOReg,
    gicd_ipriorityr: MMIOReg,
    gicd_itargetsr: MMIOReg,
    gicd_icfgr: MMIOReg,
    gicd_sgir: MMIOReg,
    gicd_cpendsgir: MMIOReg,
    gicd_spendsgir: MMIOReg,
}

pub struct GICCRegs
{
    gicc_ctlr: MMIOReg,
    gicc_pmr: MMIOReg,
    gicc_bpr: MMIOReg,
    gicc_iar: MMIOReg,
    gicc_eoir: MMIOReg,
    gicc_rpr: MMIOReg,
    gicc_hppir: MMIOReg,
    gicc_apr: MMIOReg,
    gicc_nsapr: MMIOReg,
    gicc_iidr: MMIOReg,
    gicc_dir: MMIOReg
}

pub struct GICHRegs
{
    gich_hcr: MMIOReg,
    gich_vtr: MMIOReg,
    gich_vmcr: MMIOReg,
    gich_misr: MMIOReg,
    gich_eisr0: MMIOReg,
    gich_eisr1: MMIOReg,
    gich_elsr0: MMIOReg,
    gich_elsr1: MMIOReg,
    gich_apr: MMIOReg,
    gich_lr: MMIOReg, // 0x100 ... 0x1FC
}

pub struct GICVRegs
{
    gicv_ctlr: MMIOReg,
    gicv_pmr: MMIOReg,
    gicv_bpr: MMIOReg,
    gicv_iar: MMIOReg,
    gicv_eoir: MMIOReg,
    gicv_rpr: MMIOReg,
    gicv_hppir: MMIOReg,
    gicv_apr: MMIOReg,
    gicv_nsapr: MMIOReg,
    gicv_iidr: MMIOReg,
    gicv_dir: MMIOReg,
}

impl GICDRegs
{
    pub fn new() -> Self {
        let mut retval: GICDRegs = GICDRegs {
            gicd_ctlr:     MMIOReg::new(GICD_BASE + 0x0000),
            gicd_typer:    MMIOReg::new(GICD_BASE + 0x0004),
            gicd_iidr:     MMIOReg::new(GICD_BASE + 0x0008),
            
            // vu32*
            gicd_igroupr:   MMIOReg::new(GICD_BASE + 0x0080),
            gicd_isenabler: MMIOReg::new(GICD_BASE + 0x0100),
            gicd_icenabler: MMIOReg::new(GICD_BASE + 0x0180),
            gicd_ispendr:   MMIOReg::new(GICD_BASE + 0x0200),
            gicd_icpendr:   MMIOReg::new(GICD_BASE + 0x0280),
            gicd_icdabr:    MMIOReg::new(GICD_BASE + 0x0300),
            gicd_icactiver: MMIOReg::new(GICD_BASE + 0x0280),
            
            // vu8*
            gicd_ipriorityr: MMIOReg::new(GICD_BASE + 0x0400),
            gicd_itargetsr: MMIOReg::new(GICD_BASE + 0x0800),
            
            // vu32*
            gicd_icfgr: MMIOReg::new(GICD_BASE + 0x0C00),
            
            // vu32
            gicd_sgir: MMIOReg::new(GICD_BASE + 0x0F00),
            
            // vu8*
            gicd_cpendsgir: MMIOReg::new(GICD_BASE + 0x0F20),
            gicd_spendsgir: MMIOReg::new(GICD_BASE + 0x0F20)
        };
        
        return retval;
    }
    
    pub fn enable_interrupt(&mut self, num: u16, core: u8)
    {
        let reg: u16 = num / 32;
        let bit: u16 = num % 32;
        
        let mut icpendr_reg: MMIOReg = self.gicd_icpendr.idx32(reg as u32);
        let mut isenabler_reg: MMIOReg = self.gicd_isenabler.idx32(reg as u32);
        let mut itargetsr_reg: MMIOReg = self.gicd_itargetsr.idx8(num as u32);
        let mut ipriorityr_reg: MMIOReg = self.gicd_ipriorityr.idx8(num as u32);
    
        // Enable the interrupt
        icpendr_reg |= bit!(bit);
        isenabler_reg |= bit!(bit);
        
        // Route to CPUn
        itargetsr_reg.set8(bit!(core));
        ipriorityr_reg.w8(0x7F);
    }
    
    pub fn disable_interrupt(&mut self, num: u16, core: u8)
    {
        let reg: u16 = num / 32;
        let bit: u16 = num % 32;
        
        let mut icpendr_reg: MMIOReg = self.gicd_icpendr.idx32(reg as u32);
        let mut isenabler_reg: MMIOReg = self.gicd_isenabler.idx32(reg as u32);
        let mut itargetsr_reg: MMIOReg = self.gicd_itargetsr.idx8(num as u32);
        let mut ipriorityr_reg: MMIOReg = self.gicd_ipriorityr.idx8(num as u32);
    
        // Enable the interrupt
        icpendr_reg |= bit!(bit);
        isenabler_reg |= bit!(bit);
        
        // Route to CPUn
        itargetsr_reg.unset8(bit!(core));
        ipriorityr_reg.w8(0xFF);
    }
    
    pub fn clear_interrupt(&mut self, num: u16, core: u8)
    {
        let reg: u16 = num / 32;
        let bit: u16 = num % 32;
        
        let mut icpendr_reg: MMIOReg = self.gicd_icpendr.idx32(reg as u32);

        // Enable the interrupt
        icpendr_reg |= bit!(bit);
    }
    
    pub fn disable_distribution(&mut self)
    {
        let core: u8 = get_core();
        if (core == 0)
        {
            self.gicd_ctlr.w32(0);
        }
    }
    
    pub fn enable_distribution(&mut self)
    {
        let core: u8 = get_core();
        if (core == 0)
        {
            self.gicd_ctlr.w32(1)
        }
    }
    
    pub fn init(&mut self)
    {
        for i in 0..7
        {
            let icenabler: MMIOReg = self.gicd_icenabler.idx32(i);
            let icpendr: MMIOReg = self.gicd_icpendr.idx32(i);
            let icactiver: MMIOReg = self.gicd_icactiver.idx32(i);
            let igroupr: MMIOReg = self.gicd_igroupr.idx32(i);

            icenabler.w32(0xFFFFFFFF);
            icpendr.w32(0xFFFFFFFF);
            icactiver.w32(0xFFFFFFFF);
            igroupr.w32(0x0);
        }
        
        for i in 0..32
        {
            let mut itargetsr_reg: MMIOReg = self.gicd_itargetsr.idx8(i);
            let mut ipriorityr_reg: MMIOReg = self.gicd_ipriorityr.idx8(i);
        
            itargetsr_reg.w8(1);
            ipriorityr_reg.w8(0x7F);
        }
        
        for i in 0..1
        {
            let icfgr: MMIOReg = self.gicd_icfgr.idx32(i);
            icfgr.w32(0);
        }
    }
}

impl GICCRegs
{
    pub fn new() -> Self {
        let mut retval: GICCRegs = GICCRegs {
            gicc_ctlr:  MMIOReg::new(GICC_BASE + 0x0000),
            gicc_pmr:   MMIOReg::new(GICC_BASE + 0x0004),
            gicc_bpr:   MMIOReg::new(GICC_BASE + 0x0008),
            gicc_iar:   MMIOReg::new(GICC_BASE + 0x000C),
            gicc_eoir:  MMIOReg::new(GICC_BASE + 0x0010),
            gicc_rpr:   MMIOReg::new(GICC_BASE + 0x0014),
            gicc_hppir: MMIOReg::new(GICC_BASE + 0x0018),
            
            // vu32*
            gicc_apr:   MMIOReg::new(GICC_BASE + 0x00D0),
            gicc_nsapr: MMIOReg::new(GICC_BASE + 0x00E0),
            
            // vu32
            gicc_iidr:  MMIOReg::new(GICC_BASE + 0x00FC),
            gicc_dir:   MMIOReg::new(GICC_BASE + 0x1000),
        };
        
        return retval;
    }
    
    pub fn get_hppir(&mut self) -> u32
    {
        return self.gicc_hppir.r32();
    }
    
    pub fn enable_eio(&mut self)
    {
        self.gicc_ctlr |= (bit!(0) | bit!(9));
    }
    
    pub fn enable_signaling(&mut self)
    {
        self.gicc_ctlr.w32(0x1);
    }
    
    pub fn disable_signaling(&mut self)
    {
        self.gicc_ctlr.w32(0x0);
    }
    
    pub fn mask_all(&mut self)
    {
        self.gicc_pmr.w32(0xFF);
    }
    
    pub fn unmask_all(&mut self)
    {
        self.gicc_pmr.w32(0);
    }
    
    pub fn set_bpr(&mut self, val: u32)
    {
        self.gicc_bpr.w32(val);
    }
}

impl GICHRegs
{
    pub fn new() -> Self {
        let mut retval: GICHRegs = GICHRegs {
            gich_hcr:   MMIOReg::new(GICH_BASE + 0x0000),
            gich_vtr:   MMIOReg::new(GICH_BASE + 0x0004),
            gich_vmcr:  MMIOReg::new(GICH_BASE + 0x0008),
            gich_misr:  MMIOReg::new(GICH_BASE + 0x0010),
            gich_eisr0: MMIOReg::new(GICH_BASE + 0x0020),
            gich_eisr1: MMIOReg::new(GICH_BASE + 0x0024),
            gich_elsr0: MMIOReg::new(GICH_BASE + 0x0030),
            gich_elsr1: MMIOReg::new(GICH_BASE + 0x0034),
            gich_apr:   MMIOReg::new(GICH_BASE + 0x00F0),
            
            // vu32*
            gich_lr:    MMIOReg::new(GICH_BASE + 0x0100), // 0x100 ... 0x1FC
        };
        
        return retval;
    }
    
    pub fn init(&mut self)
    {
        self.gich_hcr |= bit!(0);
        self.gich_vmcr |= bit!(0);// | BIT(9);// | BIT(9);// | BIT(9);
    }
}

impl GICVRegs
{
    pub fn new() -> Self {
        let mut retval: GICVRegs = GICVRegs {
            gicv_ctlr:  MMIOReg::new(GICV_BASE + 0x0000),
            gicv_pmr:   MMIOReg::new(GICV_BASE + 0x0004),
            gicv_bpr:   MMIOReg::new(GICV_BASE + 0x0008),
            gicv_iar:   MMIOReg::new(GICV_BASE + 0x000C),
            gicv_eoir:  MMIOReg::new(GICV_BASE + 0x0010),
            gicv_rpr:   MMIOReg::new(GICV_BASE + 0x0014),
            gicv_hppir: MMIOReg::new(GICV_BASE + 0x0018),
            gicv_apr:   MMIOReg::new(GICV_BASE + 0x00D0),
            gicv_nsapr: MMIOReg::new(GICV_BASE + 0x00E0),
            gicv_iidr:  MMIOReg::new(GICV_BASE + 0x00FC),
            gicv_dir:   MMIOReg::new(GICV_BASE + 0x1000),
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
        self.gicc.disable_signaling();
        self.gicd.disable_distribution();
        self.gicc.unmask_all();
        self.gicc.set_bpr(7);
        
        self.gicd.init();
        
        self.gicd.enable_distribution();
        self.gicc.enable_signaling();
        self.gicc.mask_all();

        // enable interrupts
        
        unsafe
        {
            asm!("msr daifclr, #0xF");
            asm!("isb");
        }
        
        //gicd_sgir = 6;
        //gicd_sgir = 6;
        //gicd_sgir = 6;
        
        self.gich.init();
        self.gicc.enable_eio();
    }
    
    pub fn enable_interrupt(&mut self, num: u16, core: u8)
    {
        self.gicd.enable_interrupt(num, core);
    }
    
    pub fn get_int_id(&mut self) -> u16
    {
        return (self.gicc.get_hppir() & 0x3FF) as u16;
    }
    
    pub fn get_int_vcpu(&mut self) -> u8
    {
        return ((self.gicc.get_hppir() >> 10) & 0x3) as u8;
    }
    
    pub fn get_rpr(&mut self) -> u8
    {
        return (self.gicc.gicc_rpr.r32() & 0xFF) as u8;
    }
    
    pub fn get_vrpr(&mut self) -> u8
    {
        return (self.gicv.gicv_rpr.r32() & 0xFF) as u8;
    }
    
    pub fn get_iar(&mut self) -> u32
    {
        return (self.gicc.gicc_iar.r32());
    }
    
    pub fn get_iar_vcpu(&mut self) -> u32
    {
        return (self.get_iar() >> 10) & 0x7;
    }
    
    pub fn get_iar_int_id(&mut self) -> u16
    {
        return (self.get_iar() & 0x3FF) as u16; // TODO IAR_IRQ_MASK
    }
    
    pub fn set_gich_vmcr(&mut self)
    {
        self.gich.gich_vmcr.or32(bit!(0));
    }
    
    pub fn set_eoir(&mut self, val: u32)
    {
        self.gicc.gicc_eoir.w32(val);
    }
    
    pub fn set_dir(&mut self, val: u32)
    {
        self.gicc.gicc_dir.w32(val);
    }
}
