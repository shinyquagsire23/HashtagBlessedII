/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::{future::Future, pin::Pin};
use crate::hos::{hport::HPort, hhandle::HHandle, hclientsession::HClientSession, hclientsession::HClientSessionHandler};
use spin::mutex::Mutex;
use crate::hos::hipc::{PKT_TYPE_INVALID, PKT_TYPE_LEGACYREQEST, PKT_TYPE_CLOSE, PKT_TYPE_LEGACYCONTROL, PKT_TYPE_REQUEST, PKT_TYPE_CONTROL, PKT_TYPE_REQUESTWITHCONTEXT, PKT_TYPE_CONTROLWITHCONTEXT, DOMAIN_CMD_SEND, DOMAIN_CMD_CLOSEOBJ};
use crate::hos::hipc::{hipc_get_handle_clientsession, hipc_get_named_serverport, hipc_register_handle_clientsession, hipc_get_packet, hipc_close_handle, hipc_register_domain, hipc_remove_domain, hipc_get_domain_session};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::prelude::v1::Box;
use crate::task::svc_wait::SvcWait;
use crate::vm::vsvc::vsvc_get_curpid_name;
use crate::hos::hdomainobj::HDomainObj;
use crate::hos::hdomainsession::HDomainSession;
use crate::modules::ipc::*;
use crate::hos::hsvc::hsvc_sleep_thread;
use crate::hos::hipc::{HObject, HObjectExtra, HExtraString};

pub fn fsp_init()
{
    ipc_register_handler(String::from("fsp-ldr"), handle_fsp_boxed);
}

async fn handle_ifile(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let mut fpath = String::from("");
    let extra = hobj.get_extra();
    match extra
    {
        HObjectExtra::String(a) => { fpath = a.str; }
        _ => {}
    }
    
    println_core!("IFile cmd {} for `{}`!", pkt.get_cmd_id(), fpath);
    
    // Call svcSleepThread before calling svcReplyAndReceive
    pre_ctx = hsvc_sleep_thread(pre_ctx, 1000).await;
    pre_ctx = hsvc_sleep_thread(pre_ctx, 1000).await;
    
    // Wait for svcReplyAndReceive to complete
    let mut post_ctx = SvcWait::new(pre_ctx).await;
    
    // Call an SVC after svcReplyAndReceive
    pre_ctx = hsvc_sleep_thread(pre_ctx, 1000).await;
    
    return post_ctx;
}


fn handle_ifile_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_ifile(pre_ctx, hobj))
}

async fn handle_ifilesystem(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    
    if pkt.get_cmd_id() == 8
    {
        let mut path = String::from("");
        if let Some(desc) = pkt.get_static(0)
        {
            path = desc.read_str();
        }
            
        // Wait for SVC to complete
        let post_ctx = SvcWait::new(pre_ctx).await;
        let resp = hipc_get_packet();
        
        // Try to hook first handle/domain if it exists
        if (resp.hook_first_handle(handle, handle_ifile_boxed))
        {
            if let Some(resp_hobj) = resp.get_first_handle_obj(handle) {
                resp_hobj.set_extra_str(&path);
            }
            println_core!("fsp-ldr::iCodeFileSystem::OpenFile(`{}`) from `{}`", path, vsvc_get_curpid_name());
        }
            
        return post_ctx;
    }
    
    return pre_ctx;
}

fn handle_ifilesystem_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_ifilesystem(pre_ctx, hobj))
}

async fn handle_fsp(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    //pkt.print();
    //println_core!("fsp-ldr cmd {} from `{}`", pkt.get_cmd_id(), vsvc_get_curpid_name());
    
    match pkt.get_cmd_id()
    {
        0 => // OpenCodeFileSystem
        {
            let tid = pkt.read_u64(0);
            let mut path = String::from("");
            if let Some(desc) = pkt.get_static(0)
            {
                path = desc.read_str();
            }
            
            // Wait for SVC to complete
            let post_ctx = SvcWait::new(pre_ctx).await;
            let resp = hipc_get_packet();
    
            // Try to hook first handle/domain if it exists
            if (resp.hook_first_handle(handle, handle_ifilesystem_boxed))
            {
                println_core!("fsp-ldr::OpenCodeFileSystem({:016x}, `{}`) from `{}`", tid, path, vsvc_get_curpid_name());
            }

            return post_ctx;
        },
        _ => { return pre_ctx; }
    }

    return pre_ctx;
}

fn handle_fsp_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_fsp(pre_ctx, hobj))
}
