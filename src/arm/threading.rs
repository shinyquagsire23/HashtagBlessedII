/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#[inline(always)]
pub fn get_core() -> u8 {
    (sysreg_read!("mpidr_el1") & 0xFF) as u8
}

#[inline(always)]
pub fn get_mpidr() -> u64 {
    sysreg_read!("mpidr_el1")
}

#[inline(always)]
pub fn get_vmpidr() -> u64 {
    sysreg_read!("vmpidr_el1")
}

#[inline(always)]
pub fn get_sp_el0() -> u64 {
    sysreg_read!("sp_el0")
}

#[inline(always)]
pub fn isb()
{
    unsafe
    {
        asm!("isb");
    }
}

#[inline(always)]
pub fn dsb()
{
    unsafe
    {
        asm!("dsb");
    }
}

#[inline(always)]
pub fn wfi()
{
    unsafe
    {
        asm!("wfi");
    }
}

#[inline(always)]
pub fn wfe()
{
    unsafe
    {
        asm!("wfe");
    }
}
