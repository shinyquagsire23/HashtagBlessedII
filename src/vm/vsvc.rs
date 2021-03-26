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
use crate::task::*;
use crate::task::svc_wait::*;
use crate::task::svc_executor::*;

use alloc::boxed::Box;
use async_trait::async_trait;

static mut LAST_CREATED: [Option<String>; 8] = [None, None, None, None, None, None, None, None];
static mut RUNNING_PROCESS_NAME: BTreeMap<u32, String> = BTreeMap::new();

include!(concat!(env!("OUT_DIR"), "/vsvc_gen.rs"));

#[async_trait]
impl SvcHandler for SvcInvalid
{
    async fn handle(&self, pre_ctx: [u64; 32]) -> [u64; 32]
    {
        panic!("Invalid SVC called!");
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcDefaultHandler
{
    async fn handle(&self, pre_ctx: [u64; 32]) -> [u64; 32]
    {
        // Pre-SVC call
        //println!("Pre-SVC call");
        
        //let post_ctx = SvcWait::new(pre_ctx).await;
        
        // Post-SVC call
        //println!("Post-SVC call");
        return pre_ctx;
    }
}

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
    let thread_ctx = peek64(translate_el1_stage12(ctx[18]));
    
    let timeout_stretch = 1;
    match svc {
        HorizonSvc::WaitSynchronization(_) => {
            ctx[3] *= timeout_stretch; // timeout
        },
        HorizonSvc::WaitProcessWideKeyAtomic(_) => {
            ctx[3] *= timeout_stretch; // timeout
        },
        HorizonSvc::WaitForAddress(_) => {
            ctx[3] *= timeout_stretch; // timeout
        },
        HorizonSvc::ReplyAndReceive(_) => {
            ctx[4] *= timeout_stretch; // timeout
        },
        HorizonSvc::ReplyAndReceiveWithUserBuffer(_) => {
            ctx[6] *= timeout_stretch; // timeout
        },
        _ => {}
    }
    
    let mut pre_ctx: [u64; 32] = Default::default();
    pre_ctx.copy_from_slice(&ctx[..32]);
    _svc_gen_pre(iss, thread_ctx, pre_ctx);
    
    // SVC handler returned early
    if let Some(ret_ctx) = task_advance_svc_ctx(thread_ctx) {
        for i in 0..32 {
            ctx[i] = ret_ctx[i];
        }
        
        return get_elr_el2();
    }
    else
    {
        // Otherwise, SVC handler is blocking for Future output
        
        let ret_ctx = SvcWait::get_ctx();
        for i in 0..32 {
            ctx[i] = ret_ctx[i];
        }
        
        return get_elr_el2();
    }

    return get_elr_el2();
}

pub fn vsvc_post_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    let thread_ctx = peek64(translate_el1_stage12(ctx[18]));
    
    let errcode = ctx[0] & 0xFFFFFFFF;
    if (errcode != 0 && errcode != 0xea01 && errcode != 0xec01 && errcode != 0xf601 && (iss & 0xFF) != 0x7F && (iss & 0xFF) != 0x7) {
        //println!("(core {}) SVC return 0x{:02x} -> {:08x}, pid {:02x} ({})", get_core(), iss & 0xFF, errcode, vsvc_get_curpid(), vsvc_get_curpid_name());
    }
    
    let mut post_ctx: [u64; 32] = Default::default();
    post_ctx.copy_from_slice(&ctx[..32]);
    SvcWait::populate_ctx(post_ctx);
    
    if let Some(ret_ctx) = task_advance_svc_ctx(thread_ctx) {
        for i in 0..32 {
            ctx[i] = ret_ctx[i];
        }
        
        return get_elr_el2();
    }
    else
    {
        // No handler, do nothing
        return get_elr_el2();
    }
}

#[async_trait]
impl SvcHandler for SvcConnectToNamedPort
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let port_name = str_from_null_terminated_utf8_u64ptr_unchecked(translate_el1_stage12(pre_ctx[1]));

        //println!("(core {}) svcConnectToNamedPort from `{}` for port {}", 
        //         get_core(), vsvc_get_curpid_name(), port_name);
        let port_name_str = String::from(port_name);
        
        //
        // Wait for SVC to complete
        //
        let post_ctx = SvcWait::new(pre_ctx).await;
        
        let session_handle = post_ctx[1] & 0xFFFFFFFF;
        
        //println!("Got session handle {:08x} for port {}", session_handle, port_name_str);
        return post_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcCreateProcess
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let proc_name = str_from_null_terminated_utf8_u64ptr_unchecked(translate_el1_stage12(pre_ctx[1]));

        //println!("(core {}) svcCreateProcess -> {}", get_core(), proc_name);
        unsafe
        {
            LAST_CREATED[get_core() as usize] = Some(String::from(proc_name));
        }
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcQueryMemory
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
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
        return pre_ctx;
    }
}
