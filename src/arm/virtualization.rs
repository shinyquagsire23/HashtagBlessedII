/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::logger::*;
use crate::arm::threading::*;

global_asm!(include_str!("virtualization.s"));

extern "C" {
    pub fn _get_cnthctl_el2() -> u64;
    pub fn _set_cnthctl_el2(val: u64);
}

pub fn get_cnthctl_el2() -> u64 {
    unsafe { return _get_cnthctl_el2(); }
}

pub fn set_cnthctl_el2(val: u64) {
    unsafe { _set_cnthctl_el2(val); }
}

pub fn timer_trap_el1()
{
    unsafe
    {
        let mut cnthctl_el2 = get_cnthctl_el2();

        //println!("{:x}", cnthctl_el2);
        cnthctl_el2 &= !0x2; // trap EL1 accesses to timer controls but not accesses
        set_cnthctl_el2(cnthctl_el2);
    }
}

pub fn timer_trap_el1_access()
{
    unsafe
    {
        let mut cnthctl_el2 = get_cnthctl_el2();

        //println!("{:x}", cnthctl_el2);
        cnthctl_el2 &= !0x3; // trap EL1 accesses to timer controls but not accesses
        set_cnthctl_el2(cnthctl_el2);
    }
}

pub fn hcr_trap_wfe()
{
    unsafe
    {
        let mut hcr_el2: u64 = 0;
        asm!("mrs {0}, hcr_el2", out(reg) hcr_el2);
        hcr_el2 |= bit!(14);   
        asm!("msr hcr_el2, {0}", in(reg) hcr_el2);
        isb();
    }
}

pub fn hcr_trap_wfi()
{
    unsafe
    {
        let mut hcr_el2: u64 = 0;
        asm!("mrs {0}, hcr_el2", out(reg) hcr_el2);
        hcr_el2 |= bit!(14);   
        asm!("msr hcr_el2, {0}", in(reg) hcr_el2);
        isb();
    }
}
