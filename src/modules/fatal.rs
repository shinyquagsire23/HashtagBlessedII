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
use crate::util::*;

pub fn fatal_init()
{
    ipc_register_handler(String::from("fatal:u"), handle_fatal_boxed);
}

async fn handle_fatal(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    match pkt.get_cmd_id()
    {
        0 => // ThrowFatal
        {
            let error = pkt.read_u32(0);
            let policy = pkt.read_u32(4);
            let tid = pkt.read_u64(8);

            println_core!("fatal::ThrowFatal(0x{:x}, 0x{:x}, 0x{:x}) from `{}`", error, policy, tid, vsvc_get_curpid_name());

            return pre_ctx;
        },
        1 => // ThrowFatalWithPolicy
        {
            let error = pkt.read_u32(0);
            let policy = pkt.read_u32(4);
            let tid = pkt.read_u64(8);
            
            println_core!("fatal::ThrowFatalWithPolicy(0x{:x}, 0x{:x}, 0x{:x}) from `{}`", error, policy, tid, vsvc_get_curpid_name());
            
            return pre_ctx;
        },
        2 => // ThrowFatalWithCpuContext
        {
            let error = pkt.read_u32(0);
            let policy = pkt.read_u32(4);
            let tid = pkt.read_u64(8);
            
            println_core!("fatal::ThrowFatalWithCpuContext(0x{:x}, 0x{:x}, 0x{:x}) from `{}`", error, policy, tid, vsvc_get_curpid_name());

            return pre_ctx;
        }
        _ => { return pre_ctx; }
    }

    return pre_ctx;
}

fn handle_fatal_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_fatal(pre_ctx, hobj))
}
