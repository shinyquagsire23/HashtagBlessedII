/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::logger::*;
use crate::arm::threading::*;

pub fn get_cnthctl_el2() -> u64 {
    sysreg_read!("cnthctl_el2")
}

pub fn set_cnthctl_el2(val: u64) {
    sysreg_write!("cnthctl_el2", val)
}

pub fn timer_trap_el1()
{
    // timer registers CVAL/TVAL/CTL_EL0 are not accessible from EL1/EL0
    sysreg_and64!("cnthctl_el2", !0x2);
}

pub fn timer_trap_el1_access()
{
    // timer registers CVAL/TVAL/CTL_EL0 are not accessible from EL1/EL0
    // CNTPCT is inaccessible from EL1/EL0
    sysreg_and64!("cnthctl_el2", !0x3);
}

pub fn hcr_trap_wfe()
{
    sysreg_or64!("hcr_el2", bit!(14));
    isb();
}

pub fn hcr_trap_wfi()
{
    sysreg_or64!("hcr_el2", bit!(13));
    isb();
}
