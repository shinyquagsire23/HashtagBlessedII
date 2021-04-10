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

pub fn set_init()
{
    ipc_register_handler(String::from("set:sys"), handle_setsys_boxed);
}

async fn handle_setsys(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    match pkt.get_cmd_id()
    {
        38 => // GetSettingsItemValue
        {
            let mut setting_id = String::from("");
            if let Some(desc) = pkt.get_static(0)
            {
                setting_id += &desc.read_str();
                setting_id += "!";
            }
            
            if let Some(desc) = pkt.get_static(1)
            {
                setting_id += &desc.read_str();
            }
            
            let recv_desc = pkt.get_recv(0);
            
            // Wait for SVC to complete
            let post_ctx = SvcWait::new(pre_ctx).await;
            let resp = hipc_get_packet();
            
            if let Some(desc) = recv_desc
            {
                let result_str = if desc.is_ascii() { desc.read_str(0) } else { format!("{:016x}", peek64(desc.get_addr_el2())) };
                //println_core!("setsys::GetSettingsItemValue(`{}`) -> `{}` from `{}`", setting_id, result_str, vsvc_get_curpid_name());
                
                if setting_id == "am.debug!force_disable_continuous_recording" {
                    poke8(desc.get_addr_el2(), 1);
                }
                /*else if setting_id == "omm!startup_fade_in_ms" {
                    poke64(desc.get_addr_el2(), 200);
                }
                else if setting_id == "omm!startup_fade_out_ms" {
                    poke64(desc.get_addr_el2(), 400);
                }
                else if setting_id == "am.debug!dev_function" {
                    poke8(desc.get_addr_el2(), 1);
                }*/
            }

            return post_ctx;
        },
        62 => // GetDebugModeFlag
        {
            pkt.write_u32(0, 1);
            return pre_ctx;
        }
        _ => { return pre_ctx; }
    }

    return pre_ctx;
}

fn handle_setsys_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_setsys(pre_ctx, hobj))
}
