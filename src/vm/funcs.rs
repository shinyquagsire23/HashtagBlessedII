/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("funcs.s"));

extern "C" {
    pub fn smc1_shim(ctx: *mut u64);
    pub fn smc0_shim(ctx: *mut u64);
    pub fn smc1(a1: u64, a2: u64, a3: u64) -> u64;
    pub fn drop_to_el1(entry: u64, arg: u64);
    pub fn vttbr_apply(ttb: *const u64);
    pub fn disable_smcstuff();
    pub fn _enable_single_step();
    pub fn _disable_single_step();
    pub fn no_hyp_stuff();
}

pub fn enable_single_step()
{
    unsafe { _enable_single_step(); }
}


pub fn disable_single_step()
{
    unsafe { _disable_single_step(); }
}

