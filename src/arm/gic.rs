/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use crate::util::*;
use crate::arm::threading::*;
use crate::vm::virq::*;
use alloc::collections::vec_deque::VecDeque;

pub const GICD_BASE: u32 = 0x50041000;
pub const GICC_BASE: u32 = 0x50042000;
pub const GICH_BASE: u32 = 0x50044000; // processor-specific 0x5000? 0x5000, 0x5200, ...
pub const GICV_BASE: u32 = 0x50046000;

pub struct GICDRegs
{
    pub gicd_ctlr: MMIOReg,
    pub gicd_typer: MMIOReg,
    pub gicd_iidr: MMIOReg,
    pub gicd_igroupr: MMIOReg,
    pub gicd_isenabler: MMIOReg,
    pub gicd_icenabler: MMIOReg,
    pub gicd_ispendr: MMIOReg,
    pub gicd_icpendr: MMIOReg,
    pub gicd_icdabr: MMIOReg,
    pub gicd_icactiver: MMIOReg,
    pub gicd_ipriorityr: MMIOReg,
    pub gicd_itargetsr: MMIOReg,
    pub gicd_icfgr: MMIOReg,
    pub gicd_sgir: MMIOReg,
    pub gicd_cpendsgir: MMIOReg,
    pub gicd_spendsgir: MMIOReg,
}

pub struct GICCRegs
{
    pub gicc_ctlr: MMIOReg,
    pub gicc_pmr: MMIOReg,
    pub gicc_bpr: MMIOReg,
    pub gicc_iar: MMIOReg,
    pub gicc_eoir: MMIOReg,
    pub gicc_rpr: MMIOReg,
    pub gicc_hppir: MMIOReg,
    pub gicc_apr: MMIOReg,
    pub gicc_nsapr: MMIOReg,
    pub gicc_iidr: MMIOReg,
    pub gicc_dir: MMIOReg
}

pub struct GICHRegs
{
    pub gich_hcr: MMIOReg,
    pub gich_vtr: MMIOReg,
    pub gich_vmcr: MMIOReg,
    pub gich_misr: MMIOReg,
    pub gich_eisr0: MMIOReg,
    pub gich_eisr1: MMIOReg,
    pub gich_elsr0: MMIOReg,
    pub gich_elsr1: MMIOReg,
    pub gich_apr: MMIOReg,
    pub gich_lr: MMIOReg, // 0x100 ... 0x1FC
}

pub struct GICVRegs
{
    pub gicv_ctlr: MMIOReg,
    pub gicv_pmr: MMIOReg,
    pub gicv_bpr: MMIOReg,
    pub gicv_iar: MMIOReg,
    pub gicv_eoir: MMIOReg,
    pub gicv_rpr: MMIOReg,
    pub gicv_hppir: MMIOReg,
    pub gicv_apr: MMIOReg,
    pub gicv_nsapr: MMIOReg,
    pub gicv_iidr: MMIOReg,
    pub gicv_dir: MMIOReg,
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
        //self.gich_hcr |= bit!(0);
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
    pub gicd: GICDRegs,
    pub gicc: GICCRegs,
    pub gich: GICHRegs,
    pub gicv: GICVRegs,
}

static mut LR_QUEUEPOSES: [u32; 8] = [0; 8];
static mut CPU_LR_QUEUES: [Option<VecDeque<u32>>; 8] = [None, None, None, None, None, None, None, None];

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
        
        if (get_core() == 0) {
            unsafe
            {
                for i in 0..8
                {
                    CPU_LR_QUEUES[i] = Some(VecDeque::new());
                }
            }
        }
    }
    
    pub fn enable_interrupt(&mut self, num: u16, core: u8)
    {
        self.gicd.enable_interrupt(num, core);
    }
    
    pub fn disable_interrupt(&mut self, num: u16, core: u8)
    {
        self.gicd.disable_interrupt(num, core);
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
    
    pub fn get_iar_vcpu(&mut self) -> u8
    {
        return ((self.get_iar() >> 10) & 0x7) as u8;
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
    
    pub fn disable_distribution(&mut self)
    {
        self.gicd.disable_distribution();
    }
    
    pub fn enable_distribution(&mut self)
    {
        self.gicd.enable_distribution();
    }
    
    pub fn find_lr_slot(&mut self) -> u8
    {
        unsafe
        {
        // find an open slot
        let virq_status = self.gich.gich_elsr0.r32();
        for i in 0..4 // TODO read this max val
        {
            if ((virq_status & bit!(i)) != 0)
            {
                return i as u8;
            }
        }

        return LR_INVALID_SLOT;
        }
    }
    
    pub fn process_queue(&mut self)
    {
        unsafe
        {
        // clear EOIs
        let mut virq_status = self.gich.gich_eisr0.r32();
        
        for i in 0..4
        {
            if ((virq_status & bit!(i)) != 0)
            {
                let lr_iter_val = self.gich.gich_lr.idx32(i).r32();
                let lr_state = (lr_iter_val >> 28) & 3;
                let lr_vid = (lr_iter_val & LR_IRQ_MASK);
                let lr_cpu = (lr_iter_val >> 10) & 3;
                
                self.set_dir(lr_vid | (lr_cpu << 10));
                
                //println!("Cleared {:x}", hwint_id);
                self.gich.gich_lr.idx32(i).w32(0);
            }
        }

        let mut lr_slot = self.find_lr_slot();
        
        let queue = CPU_LR_QUEUES[get_core() as usize].as_mut().unwrap();

        while (!queue.is_empty() && lr_slot != LR_INVALID_SLOT)
        {
            let lr_val = queue.pop_front().unwrap();

            // make sure we're not sending a request that is already pending
            for i in 0..4
            {
                let lr_iter_val = self.gich.gich_lr.idx32(i).r32();
                let lr_state = (lr_iter_val >> 28) & 3;
                let lr_vid = (lr_iter_val & 0x3FF) as u16;
                let lr_cpu = (lr_iter_val >> 10) & 3;
                let lr_prio = (lr_iter_val >> LR_PRIO_SHIFT) & LR_PRIO_MASK;

                let lrval_prio = (lr_val >> LR_PRIO_SHIFT) & LR_PRIO_MASK;
                let lrval_cpu = (lr_val >> 10) & 3;
                let int_id = (lr_val & 0x3FF) as u16;
                if (tegra_irq_is_sgi(int_id) && lr_vid == int_id && lr_cpu == lrval_cpu) // | pending bit if only active?
                {
                    println!("Didn't push IRQ {}!", int_id);
                    lr_slot = LR_INVALID_SLOT;
                    break;
                }
                else if (!tegra_irq_is_sgi(int_id) && lr_vid == int_id && (lr_state != LR_STS_INVALID))
                {
                    println!("Didn't push IRQ {}!", int_id);
                    lr_slot = LR_INVALID_SLOT;
                    break;
                }

                /*if (lr_vid == int_id && (lr_state != LR_STS_INVALID))
                {
                    lr_slot = LR_INVALID_SLOT; // don't queue another interrupt if there's one pending
                    break;
                }*/
            }
            
            if (lr_slot >= 4) 
            {
                lr_slot = LR_INVALID_SLOT;
            }
            
            if (lr_slot == LR_INVALID_SLOT)
            {
                queue.push_front(lr_val);
                break;
            }

            self.gich.gich_lr.idx32(lr_slot as u32).w32(lr_val);

            lr_slot = self.find_lr_slot();
        }

        let mut taken_slots: u8 = 0;
        virq_status = self.gich.gich_eisr0.r32();
        for i in 0..4
        {
            if ((virq_status & bit!(i)) != 0) {
                taken_slots += 1;
            }
        }

        // didn't deplete the queue and we still have entries,
        // IRQ on no pending to make sure the queue gets handled
        if (!queue.is_empty())
        {
            self.gich.gich_hcr |= GICH_INT_U;
        }
        }
    }
    
    pub fn do_maintenance(&mut self)
    {
        if (self.gich.gich_misr.bits_set(GICH_INT_NP))
        {
            self.gich.gich_hcr &= !GICH_INT_NP;
            println!("no pending {:x} {:08x} {:08x}", self.gich.gich_eisr0.r32(), self.gich.gich_lr.idx32(0).r32(), self.gich.gich_lr.idx32(1).r32());
            //GICH_HCR |= GICH_INT_U;
        }
        else if (self.gich.gich_misr.bits_set(GICH_INT_U))
        {
            println!("underflow {:x} {:08x} {:08x}", self.gich.gich_eisr0.r32(), self.gich.gich_lr.idx32(0).r32(), self.gich.gich_lr.idx32(1).r32());

            self.gich.gich_hcr &= !GICH_INT_U;
        }
        else if (self.gich.gich_misr.bits_set(GICH_INT_EOI)) // EOI
        {
            println!("eoi {:08x} {:x} {:08x} {:08x}", self.gich.gich_hcr.r32(), self.gich.gich_eisr0.r32(), self.gich.gich_lr.idx32(0).r32(), self.gich.gich_lr.idx32(1).r32());
        }

        self.process_queue();
    }
    
    pub fn send_interrupt(&mut self, int_id: u16, vcpu: u8, prio: u8)
    {
        //println!("(core {}) Sending int_id {} to vcpu {}, prio {}", get_core(), int_id, vcpu, prio);

        let mut lr_val = 0;
        if (!tegra_irq_is_sgi(int_id)) // hwint
        {
            lr_val |= LR_HWINT;
            lr_val |= (LR_STS_PENDING << LR_STS_SHIFT);
            lr_val |= ((int_id as u32) << LR_SHIFT_PIRQ); // physical IRQ id, sent to the distributer on vEOIR
            lr_val |= ((int_id as u32) << LR_SHIFT_VIRQ); // virtual IRQ id, sent to the vCPU
        }
        else
        {
            lr_val |= (LR_STS_PENDING << LR_STS_SHIFT);
            lr_val |= ((vcpu as u32) << LR_SHIFT_VCPU);
            lr_val |= ((int_id as u32) << LR_SHIFT_VIRQ);
            lr_val |= bit!(19);
        }

        // make sure we're not sending a request that is already pending
        for i in 0..4
        {
            let lr_iter_val = self.gich.gich_lr.idx32(i).r32();
            let lr_state = (lr_iter_val >> 28) & 3;
            let lr_vid = (lr_iter_val & 0x3FF) as u16;
            let lr_cpu = ((lr_iter_val >> 10) & 3) as u8;
            let lr_prio = (lr_iter_val >> LR_PRIO_SHIFT) & LR_PRIO_MASK;


            let lrval_prio = (lr_val >> LR_PRIO_SHIFT) & LR_PRIO_MASK;
            if (tegra_irq_is_sgi(int_id) && lr_vid == int_id && lr_cpu == vcpu && (lr_state == LR_STS_PENDING))
            {
                println!("Tried to queue SGI {:x} to core {} while pending", int_id, vcpu);
                return; // don't queue another interrupt if there's one pending
            }
            else if (!tegra_irq_is_sgi(int_id) && lr_vid == int_id && (lr_state != LR_STS_INVALID))
            {
                println!("Tried to queue {:x} while pending", int_id);
                return; // don't queue another interrupt if there's one pending
            }
            
            /*else if (lr_vid == int_id )
            {
                println!("Tried to queue {:x} while pending", int_id);
                return; // don't queue another interrupt if there's one pending
            }*/
        }
        
        unsafe
        {
        let queue_iter = CPU_LR_QUEUES[get_core() as usize].as_ref().unwrap().iter();
        for _lr_iter_val in queue_iter
        {
            let lr_iter_val = _lr_iter_val;
            let lr_state = (lr_iter_val >> 28) & 3;
            let lr_vid = (lr_iter_val & 0x3FF) as u16;
            let lr_cpu = ((lr_iter_val >> 10) & 3) as u8;
            let lr_prio = (lr_iter_val >> LR_PRIO_SHIFT) & LR_PRIO_MASK;


            let lrval_prio = (lr_val >> LR_PRIO_SHIFT) & LR_PRIO_MASK;
            if (tegra_irq_is_sgi(int_id) && lr_vid == int_id && lr_cpu == vcpu && (lr_state == LR_STS_PENDING))
            {
                println!("Tried to queue SGI {:x} to core {} while pending", int_id, vcpu);
                return; // don't queue another interrupt if there's one pending
            }
            else if (!tegra_irq_is_sgi(int_id) && lr_vid == int_id && (lr_state != LR_STS_INVALID))
            {
                println!("Tried to queue {:x} while pending", int_id);
                return; // don't queue another interrupt if there's one pending
            }
        }

        let queue = CPU_LR_QUEUES[get_core() as usize].as_mut().unwrap();
        queue.push_back(lr_val);
        }
    }
    
    pub fn get_gich_misr(&mut self) -> u32
    {
        self.gich.gich_misr.r32()
    }
}
