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

pub fn timerGetTick() -> u32 {
    return peek32(TIMERUS_CNTR_1US_ADDR);
}

pub fn timerWait(uSecs: u32)
{
    let end: u64 = (peek32(TIMERUS_CNTR_1US_ADDR) as u64 + uSecs as u64);
    
    loop {
        if ((peek32(TIMERUS_CNTR_1US_ADDR) as u64) < end) { break };
    }
}
