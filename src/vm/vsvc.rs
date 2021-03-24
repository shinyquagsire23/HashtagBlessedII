/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::arm::exceptions::*;
use crate::arm::threading::*;
use crate::arm::mmu::*;
use crate::vm::funcs::*;
use crate::hos::svc::*;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use crate::util::*;
use alloc::sync::Arc;

static mut LAST_CREATED: [Option<String>; 8] = [None, None, None, None, None, None, None, None];
static mut RUNNING_PROCESS_NAME: BTreeMap<u32, String> = BTreeMap::new();

pub fn vsvc_init()
{
    // 8.0.1, TODO parse this from KIPs?
    unsafe
    {
        RUNNING_PROCESS_NAME.insert(1, String::from("FS"));
        RUNNING_PROCESS_NAME.insert(2, String::from("Loader"));
        RUNNING_PROCESS_NAME.insert(3, String::from("NCM"));
        RUNNING_PROCESS_NAME.insert(4, String::from("ProcessMana"));
        RUNNING_PROCESS_NAME.insert(5, String::from("sm"));
        RUNNING_PROCESS_NAME.insert(6, String::from("spl"));
        RUNNING_PROCESS_NAME.insert(7, String::from("boot"));
        RUNNING_PROCESS_NAME.insert(0xFF, String::from("idle core"));
    }
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

pub fn vsvc_get_pid_name(pid: u32) -> String
{
    unsafe
    {
        match RUNNING_PROCESS_NAME.get(&pid) {
           Some(name) => name.clone(),
           None => String::from("unknown pid")
        }
    }
}

pub fn vsvc_get_curpid_name() -> String
{
    let pid = (vsvc_get_curpid() & 0xFF) as u32;
    return vsvc_get_pid_name(pid);
}

pub fn vsvc_pre_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    let svc = HorizonSvc::from_iss(iss);
    let timeout_stretch = 1;
    match svc {
        HorizonSvc::WaitSynchronization => {
            ctx[3] *= timeout_stretch; // timeout
        },
        HorizonSvc::WaitProcessWideKeyAtomic => {
            ctx[3] *= timeout_stretch; // timeout
        },
        HorizonSvc::WaitForAddress => {
            ctx[3] *= timeout_stretch; // timeout
        },
        HorizonSvc::ReplyAndReceive => {
            ctx[4] *= timeout_stretch; // timeout
        },
        HorizonSvc::ReplyAndReceiveWithUserBuffer => {
            ctx[6] *= timeout_stretch; // timeout
        },
        HorizonSvc::CreateProcess => {
            let proc_name = str_from_null_terminated_utf8_u64ptr_unchecked(translate_el1_stage12(ctx[1]));

            //println!("(core {}) svcCreateProcess -> {}", get_core(), proc_name);
            unsafe
            {
                LAST_CREATED[get_core() as usize] = Some(String::from(proc_name));
            }
        },
        HorizonSvc::QueryMemory => {
            unsafe
            {
                if !RUNNING_PROCESS_NAME.contains_key(&vsvc_get_curpid())
                {
                    let name = &LAST_CREATED[get_core() as usize];
                    if name.is_some() {
                        RUNNING_PROCESS_NAME.insert(vsvc_get_curpid(), name.as_ref().unwrap().clone());
                    }
                }
            }
        },
        _ => {}
    }
    return get_elr_el2();
}

pub fn vsvc_post_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    //println!("SVC post");
    if (get_core() == 3 && vsvc_get_curpid() == 1) {
        //enable_single_step();
        //ctx[38] |= (1<<21);
    }
    return get_elr_el2();
}
