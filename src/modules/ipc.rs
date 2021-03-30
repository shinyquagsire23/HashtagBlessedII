/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::{future::Future, pin::Pin};
use crate::hos::{hport::HPort, hhandle::HHandle, hclientsession::HClientSession};
use spin::mutex::Mutex;
use crate::hos::hipc::{hipc_get_handle_clientsession, hipc_get_named_serverport, hipc_register_handle_clientsession, hipc_get_packet};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::prelude::v1::Box;
use crate::task::svc_wait::SvcWait;
use crate::vm::vsvc::vsvc_get_curpid_name;

pub fn ipc_init()
{
    
}

async fn handle_fsp(mut pre_ctx: [u64; 32]) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    //pkt.print();
    /*if pkt.is_domain()
    {
        pkt.print();
    }*/
    println_core!("fsp-ldr cmd {} from `{}`", pkt.get_cmd_id(), vsvc_get_curpid_name());    
    
    return pre_ctx;
}

fn handle_fsp_boxed(mut pre_ctx: [u64; 32]) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_fsp(pre_ctx))
}

async fn handle_sm(mut pre_ctx: [u64; 32]) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    //pkt.print();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let sm_hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    match pkt.get_cmd_id()
    {
        0 => // RegisterClient
        { 
            
        },
        1 => // GetServiceHandle
        { 
            let name = pkt.read_str(0);
            
            // Wait for SVC to complete
            let post_ctx = SvcWait::new(pre_ctx).await;
            let resp = hipc_get_packet();
            
            if let Some(handle) = resp.get_handle(0) {
                //println_core!("sm::GetServiceHandle(`{}`) -> {:x}", name, handle);
                
                // TODO: Copied handles may not actually belong to parent
                let mut service_hsession = sm_hsession.lock().new_from_parent();
                
                // Set handler
                if name == "fsp-ldr" {
                    service_hsession.set_handler(handle_fsp_boxed);
                }
                
                // Link new HClientSession to HOS handle
                hipc_register_handle_clientsession(handle, Arc::new(Mutex::new(service_hsession)));
            }
            
            return post_ctx;
        },
        2 => // RegisterService
        { 
            let name = pkt.read_str(0);
            println_core!("sm::RegisterService(`{}`) from `{}`", name, vsvc_get_curpid_name());
        },
        3 => // UnregisterService
        { 
            let name = pkt.read_str(0);
            println_core!("sm::UnregisterService(`{}`) from `{}`", name, vsvc_get_curpid_name());
        },
        4 => // DetachClient (11.0.0+)
        { 
            
        },
        _ => {}
    }
    
    return pre_ctx;
}

fn handle_sm_boxed(mut pre_ctx: [u64; 32]) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_sm(pre_ctx))
}

pub fn ipc_hook_namedport(port_name_str: &String, session_handle: u32)
{
    // Get port struct
    if let Some(hport) = hipc_get_named_serverport(port_name_str)
    {
        let mut hsession = hport.lock().create_session();
        
        // Set handler
        if port_name_str == "sm:" {
            hsession.set_handler(handle_sm_boxed);
        }

        // Link new HClientSession to HOS handle
        hipc_register_handle_clientsession(session_handle, Arc::new(Mutex::new(hsession)));
    }
}
