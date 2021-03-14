/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("threading.s"));

extern "C" {
    pub fn get_core() -> u32;
    pub fn get_core2() -> u32;
    pub fn get_mpidr() -> u64;
    pub fn get_vmpidr() -> u64;
    pub fn getSP_EL0() -> u64;
}
