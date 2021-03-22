/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("exceptions.s"));

extern "C" {
    pub fn _get_elr_el2() -> u64;
    pub fn _get_esr_el2() -> u32;
    pub fn _get_afsr0_el2() -> u32;
    pub fn _get_afsr1_el2() -> u32; 
    pub fn _get_sp_el2() -> u64;

    pub fn _get_elr_el1() -> u64;
    pub fn _get_esr_el1() -> u32;
    pub fn _get_afsr0_el1() -> u32;
    pub fn _get_afsr1_el1() -> u32; 
    pub fn _get_sp_el0() -> u64;
    pub fn _get_sp_el1() -> u64;
    pub fn _get_hpfar_el2() -> u64;
    pub fn _get_far_el2() -> u64;

}

#[inline(always)]
pub fn get_fipa_el2() -> u64
{
    unsafe
    {
        return (_get_hpfar_el2() << 8) | (_get_far_el2() & 0xFFF);
    }
}

#[inline(always)]
pub fn get_elr_el2() -> u64
{
    unsafe
    {
        return _get_elr_el2();
    }
}

#[inline(always)]
pub fn get_esr_el2() -> u32
{
    unsafe
    {
        return _get_esr_el2();
    }
}

#[inline(always)]
pub fn get_elr_el1() -> u64
{
    unsafe
    {
        return _get_elr_el1();
    }
}

#[inline(always)]
pub fn get_esr_el1() -> u32
{
    unsafe
    {
        return _get_esr_el1();
    }
}

#[inline(always)]
pub fn get_afsr0_el1() -> u32
{
    unsafe
    {
        return _get_afsr0_el1();
    }
}

#[inline(always)]
pub fn get_afsr1_el1() -> u32
{
    unsafe
    {
        return _get_afsr1_el1();
    }
}
