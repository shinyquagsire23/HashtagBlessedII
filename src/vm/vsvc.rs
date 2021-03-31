/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::arm::exceptions::*;
use crate::arm::threading::*;
use crate::arm::mmu::translate_el1_stage12;
use crate::vm::funcs::*;
use crate::hos::svc::*;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use crate::util::*;
use alloc::sync::Arc;
use crate::task::*;
use crate::task::svc_wait::*;
use crate::task::svc_executor::*;
use crate::logger::log_cmd;
use alloc::vec::Vec;
use core::{future::Future, pin::Pin};
use crate::hos::{hipc::*, hport::HPort, hhandle::HHandle, hclientsession::HClientSession, hclientsession::HClientSessionHandler};
use spin::mutex::Mutex;
use crate::modules::ipc::{ipc_handle_syncrequest, ipc_hook_namedport};

use alloc::boxed::Box;
use async_trait::async_trait;

static mut LAST_CREATED: [Option<String>; 8] = [None, None, None, None, None, None, None, None];
static mut RUNNING_PROCESS_NAME: BTreeMap<u32, String> = BTreeMap::new();
static mut PROCESS_NAME_PID: BTreeMap<String, u32> = BTreeMap::new();
static mut VSVC_QLAUNCH_STARTED: bool = false;
static mut VSVC_TTBRS: BTreeMap<u32, u64> = BTreeMap::new();
static mut VSVC_PROC_HANDLES: BTreeMap<u32, String> = BTreeMap::new();

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

pub fn vsvc_register_ttbr(pid: u32, ttbr: u64)
{
    unsafe
    {
        VSVC_TTBRS.insert(pid, ttbr);
    }
}

pub fn vsvc_get_pid_ttbr(pid: u32) -> u64
{
    unsafe
    {
        match VSVC_TTBRS.get(&pid) {
           Some(addr) => *addr,
           None => 0
        }
    }
}

pub fn vsvc_is_qlaunch_started() -> bool
{
    unsafe { return VSVC_QLAUNCH_STARTED; }
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
        
        PROCESS_NAME_PID.insert(String::from("FS"), 1);
        PROCESS_NAME_PID.insert(String::from("Loader"), 2);
        PROCESS_NAME_PID.insert(String::from("NCM"), 3);
        PROCESS_NAME_PID.insert(String::from("ProcessMana"), 4);
        PROCESS_NAME_PID.insert(String::from("sm"), 5);
        PROCESS_NAME_PID.insert(String::from("spl"), 6);
        PROCESS_NAME_PID.insert(String::from("boot"), 7);
        PROCESS_NAME_PID.insert(String::from("kernel"), 0xFF);
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

pub fn vsvc_get_pid_list() -> Vec<u32>
{
    unsafe
    {
        return RUNNING_PROCESS_NAME.keys().cloned().collect();
    }
}

pub fn vsvc_get_pid_name(pid: u32) -> String
{
    unsafe
    {
        match RUNNING_PROCESS_NAME.get(&pid) {
           Some(name) => name.clone(),
           None => format!("unknown pid {}", pid)
        }
    }
}

pub fn vsvc_get_process_pid(name: &String) -> u32
{
    unsafe
    {
        match PROCESS_NAME_PID.get(name) {
           Some(pid) => *pid,
           None => 0
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
    //let svc = HorizonSvc::from_iss(iss);
    let thread_ctx = peek64(translate_el1_stage12(ctx[18]));
    
    /*let timeout_stretch = 1;
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
    }*/
    
    let mut pre_ctx: [u64; 32] = Default::default();
    pre_ctx.copy_from_slice(&ctx[..32]);
    if _svc_gen_pre(iss, thread_ctx, pre_ctx) {
        return get_elr_el2();
    }
    
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
impl SvcHandler for SvcManageNamedPort
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let port_name = kstr!(pre_ctx[1]);
        let max_sessions = (pre_ctx[2] & 0xFFFFFFFF) as u32;

        println_core!("svcManageNamedPort from `{}` for port `{}`", 
                 vsvc_get_curpid_name(), port_name);
        let port_name_str = String::from(port_name);
        
        //
        // Wait for SVC to complete
        //
        let post_ctx = SvcWait::new(pre_ctx).await;
        
        // TODO error handling
        
        let port_handle = (post_ctx[1] & 0xFFFFFFFF) as u32;
        
        let hport = HPort::from_curpid(Some(port_name_str));
        hipc_register_handle_serverport(port_handle, Arc::new(Mutex::new(hport)));
        
        //println!("Got session handle {:08x} for port {}", session_handle, port_name_str);
        return post_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcConnectToNamedPort
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let port_name = kstr!(pre_ctx[1]);

        println_core!("svcConnectToNamedPort from `{}` for port {}", 
                 vsvc_get_curpid_name(), port_name);
        
        let port_name_str = String::from(port_name);

        //
        // Wait for SVC to complete
        //
        let post_ctx = SvcWait::new(pre_ctx).await;
        
        let session_handle = (post_ctx[1] & 0xFFFFFFFF) as u32;
        ipc_hook_namedport(&port_name_str, session_handle);
        
        //println!("Got session handle {:08x} for port {}", session_handle, port_name_str);
        return post_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcBreak
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        println_core!("process `{}` (pid {}) called svcBreak!", vsvc_get_curpid_name(), vsvc_get_curpid());

        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcOutputDebugString
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let str_len = (pre_ctx[1] & 0xFFFFFFFF) as u32;
        let debug_str = kstr_len!(pre_ctx[0], str_len);
        println_core!("svcOutputDebugString({}): {}", vsvc_get_curpid_name(), debug_str);
        
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcSleepSystem
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        println_uarta!("svcSleepSystem({}) STUB", vsvc_get_curpid_name());
        crate::io::timer::timer_wait(1000000);
        
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcCreateProcess
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let proc_name = kstr!(pre_ctx[1]);

        //
        // Wait for SVC to complete
        //
        let post_ctx = SvcWait::new(pre_ctx).await;
        
        let process_handle = (post_ctx[1] & 0xFFFFFFFF) as u32;
        
        println_core!("svcCreateProcess from `{}` -> {} (handle {:x})", vsvc_get_curpid_name(), proc_name, process_handle);
        unsafe
        {
            LAST_CREATED[get_core() as usize] = Some(String::from(proc_name));
            
            if (proc_name == "overlayDisp") {
                VSVC_QLAUNCH_STARTED = true;
            }
        }

        return post_ctx;
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
                    PROCESS_NAME_PID.insert(name.as_ref().unwrap().clone(), vsvc_get_curpid());
                }
            }
        }
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcExitProcess
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        unsafe
        {
            let pid = vsvc_get_curpid();
            if RUNNING_PROCESS_NAME.contains_key(&pid)
            {
                if let Some(name) = RUNNING_PROCESS_NAME.get(&pid) {
                    println_core!("svcExitProcess -> {}", name);
                   PROCESS_NAME_PID.remove(name);
                }
        
                RUNNING_PROCESS_NAME.remove(&pid);
            }
        }
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcStartProcess
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let process_handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
        println_core!("svcStartProcess from {} for handle {:x}", vsvc_get_curpid_name(), process_handle);
        
        unsafe
        {
            if let Some(proc_name) = &LAST_CREATED[get_core() as usize] {
                VSVC_PROC_HANDLES.insert(process_handle, proc_name.clone());
            }
        }
        
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcTerminateProcess
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
        println_core!("svcTerminateProcess from {} for handle {:x}", vsvc_get_curpid_name(), handle);
        
        unsafe
        {
            if let Some(proc_name) = VSVC_PROC_HANDLES.remove(&handle) 
            {
                println!("    -> Terminated process {}", proc_name);
                if let Some(pid) = PROCESS_NAME_PID.remove(&proc_name) {
                    RUNNING_PROCESS_NAME.remove(&pid);
                }
            }
        }
        
        // TODO check error?

        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcReplyAndReceive
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        return pre_ctx;
    }
}

#[async_trait]
impl SvcHandler for SvcSendSyncRequest
{
    async fn handle(&self, mut pre_ctx: [u64; 32]) -> [u64; 32]
    {
        return ipc_handle_syncrequest(pre_ctx).await;
    }
}
