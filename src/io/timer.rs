/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![allow(warnings, unused)]

use crate::util::*;

pub const TMR_PADDR : u32 = (0x60005000);
pub const TMR_VADDR : u32 = (0x60005000);

pub const TIMERUS_CNTR_1US_ADDR : u32 = (TMR_VADDR + 0x010);

pub fn timer_get_tick() -> u32 {
    return peek32(TIMERUS_CNTR_1US_ADDR);
}

pub fn timer_wait(uSecs: u32)
{
    let read_init = timer_get_tick();
    let mut end: u64 = (read_init as u64) + (uSecs as u64);

    if (end > 0x100000000)
    {
        end -= 0x100000000;
        loop {
            if (timer_get_tick() < read_init) { break };
        }
    }
    
    loop {
        if (((timer_get_tick() as u64) & 0xFFFFFFFF) >= end) { break };
    }
}
