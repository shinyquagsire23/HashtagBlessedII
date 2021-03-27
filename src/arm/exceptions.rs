/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#[inline(always)]
pub fn get_fipa_el2() -> u64
{
    unsafe
    {
        return (get_hpfar_el2() << 8) | (get_far_el2() & 0xFFF);
    }
}

#[inline(always)]
pub fn get_far_el2() -> u64
{
    sysreg_read!("far_el2")
}

#[inline(always)]
pub fn get_hpfar_el2() -> u64
{
    sysreg_read!("hpfar_el2")
}

#[inline(always)]
pub fn get_elr_el2() -> u64
{
    sysreg_read!("elr_el2")
}

#[inline(always)]
pub fn get_esr_el2() -> u32
{
    sysreg_read!("esr_el2") as u32
}

#[inline(always)]
pub fn get_elr_el1() -> u64
{
    sysreg_read!("elr_el1")
}

#[inline(always)]
pub fn get_esr_el1() -> u32
{
    sysreg_read!("esr_el1") as u32
}

#[inline(always)]
pub fn get_afsr0_el1() -> u32
{
    sysreg_read!("afsr0_el1") as u32
}

#[inline(always)]
pub fn get_afsr1_el1() -> u32
{
    sysreg_read!("afsr1_el1") as u32
}

#[inline(always)]
pub fn get_spsr_el2() -> u64
{
    sysreg_read!("spsr_el2")
}

#[inline(always)]
pub fn get_spsr_el1() -> u64
{
    sysreg_read!("spsr_el1")
}
