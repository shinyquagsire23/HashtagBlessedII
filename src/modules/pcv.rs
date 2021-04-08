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

pub fn pcv_init()
{
    ipc_register_handler(String::from("clkrst"), handle_clkrst_boxed);
    ipc_register_handler(String::from("clkrst:i"), handle_clkrst_boxed);
}

async fn handle_clksession(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let mut dev = 0xFFFFFFFF;
    let extra = hobj.get_extra();
    match extra
    {
        HObjectExtra::U32(a) => { dev = a.val; }
        _ => {}
    }
    
    if pkt.get_cmd_id() == 7
    {
        let mut hz = pkt.read_u32(0);
        println_core!("clkrst::iClkrstSession::SetClockRate(dev_id={:x}, hz={}) from `{}`", dev, hz, vsvc_get_curpid_name());
        
        // CPU clocks
        if dev == 0x40000001 {
            hz = 1785 * 1000000;
            println_core!("clkrst: Overclocking to 1.785GHz!");
        }
        
        pkt.write_u32(0, hz);

        return pre_ctx;
    }
    else
    {
        //println_core!("clkrst::iClkrstSession::Cmd{}(dev_id={:x}) from `{}`", pkt.get_cmd_id(), dev, vsvc_get_curpid_name());
    }
    
    return pre_ctx;
}

fn handle_clksession_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_clksession(pre_ctx, hobj))
}

async fn handle_clkrst(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    match pkt.get_cmd_id()
    {
        0 => // OpenSession
        {
            let dev = pkt.read_u32(0);
            
            // Wait for SVC to complete
            let post_ctx = SvcWait::new(pre_ctx).await;
            let resp = hipc_get_packet();
    
            // Try to hook first handle/domain if it exists
            if (resp.hook_first_handle(handle, handle_clksession_boxed))
            {
                if let Some(resp_hobj) = resp.get_first_handle_obj(handle) {
                    resp_hobj.set_extra_u32(dev);
                }
                //println_core!("clkrst::OpenSession({:08x}) from `{}`", dev, vsvc_get_curpid_name());
            }

            return post_ctx;
        },
        _ => { return pre_ctx; }
    }

    return pre_ctx;
}

fn handle_clkrst_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_clkrst(pre_ctx, hobj))
}
