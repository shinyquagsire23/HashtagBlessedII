/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("funcs.s"));

extern "C" {
    pub fn smc1_shim(ctx: u64);
    pub fn smc0_shim(ctx: u64);
    pub fn smc1(a1: u64, a2: u64, a3: u64) -> u64;
    pub fn drop_to_el1(entry: u64);
    pub fn vttbr_init(ttb_addr: u64);
    pub fn disable_smcstuff();
}

