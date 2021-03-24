/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::arm::exceptions::*;
use crate::arm::threading::*;
use crate::vm::funcs::*;

pub fn vsvc_init()
{

}

pub fn vsvc_get_curpid() -> u32
{
    unsafe
    {
        let mut contextidr: u64 = 0;
        asm!("mrs {0}, CONTEXTIDR_EL1", out(reg) contextidr);
        let pid = (contextidr & 0xFF) as u32;

        return pid;
    }
}

pub const fn vsvc_get_pid_name<'a>(pid: &'a u8) -> &'a str
{
    return ""; //TODO
}

pub const fn vsvc_get_curpid_name() -> &'static str
{
    return ""; //TODO
}

pub fn vsvc_pre_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    let svc_num = iss & 0xFF;
    println!("(core {}) SVC 0x{:02x}, pid {:02x}", get_core(), svc_num, vsvc_get_curpid());
    if (get_core() == 3 && vsvc_get_curpid() == 1) {
        enable_single_step();
        ctx[38] |= (1<<21);
    }
    return get_elr_el2();
}

pub fn vsvc_post_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    //println!("SVC post");
    return get_elr_el2();
}
