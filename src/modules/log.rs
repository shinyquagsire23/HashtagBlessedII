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

pub fn log_init()
{
    ipc_register_handler(String::from("lm"), handle_lm_boxed);
}

async fn handle_logger(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    //pkt.print();
    
    if pkt.get_cmd_id() == 0
    {
        let mut logged = String::from("");
        if let Some(desc) = pkt.get_static(0)
        {
            let addr = desc.get_addr_el2();
            let payload_size = peek32(addr + 0x14);
            
            let mut payload = addr + 0x18;
            let payload_end = addr + 0x18 + payload_size as u64;
            while payload < payload_end
            {
                let chunk_key = peek8(payload);
                let chunk_len = peek8(payload+1);
                if chunk_key == 2
                {
                    logged += hypstr_len!(payload+2, chunk_len);
                }
                payload += (2 + chunk_len as u64);
            }
        }
        println_core!("lm::iLogger::Log(`{}`) from `{}`", logged, vsvc_get_curpid_name());

        return pre_ctx;
    }
    else
    {
        //println_core!("clkrst::iLogger::Cmd{}(dev_id={:x}) from `{}`", pkt.get_cmd_id(), dev, vsvc_get_curpid_name());
    }
    
    return pre_ctx;
}

fn handle_logger_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_logger(pre_ctx, hobj))
}

async fn handle_lm(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    match pkt.get_cmd_id()
    {
        0 => // OpenLogger
        {
            // Wait for SVC to complete
            let post_ctx = SvcWait::new(pre_ctx).await;
            let resp = hipc_get_packet();
    
            // Try to hook first handle/domain if it exists
            if (resp.hook_first_handle(handle, handle_logger_boxed))
            {
                //println_core!("lm::OpenLogger() from `{}`", vsvc_get_curpid_name());
            }

            return post_ctx;
        },
        _ => { return pre_ctx; }
    }

    return pre_ctx;
}

fn handle_lm_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_lm(pre_ctx, hobj))
}
