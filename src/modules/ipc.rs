/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::{future::Future, pin::Pin};
use crate::hos::{hport::HPort, hhandle::HHandle, hclientsession::HClientSession};
use spin::mutex::Mutex;
use crate::hos::hipc::{hipc_get_named_serverport, hipc_register_handle_clientsession, hipc_get_packet};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::prelude::v1::Box;
use crate::task::svc_wait::SvcWait;

pub fn ipc_init()
{
    
}

async fn handle_sm(mut pre_ctx: [u64; 32]) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    pkt.print();
    
    //
    // Wait for SVC to complete
    //
    let post_ctx = SvcWait::new(pre_ctx).await;
    let resp = hipc_get_packet();
    resp.print();
    
    return post_ctx;
            
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
        
        if port_name_str == "sm:" {
            hsession.set_handler(handle_sm_boxed);
        }

        hipc_register_handle_clientsession(session_handle, Arc::new(Mutex::new(hsession)));
    }
}
