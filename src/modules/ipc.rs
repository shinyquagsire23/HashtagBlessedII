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
use alloc::collections::BTreeMap;
use alloc::prelude::v1::Box;
use crate::task::svc_wait::SvcWait;
use crate::vm::vsvc::vsvc_get_curpid_name;
use crate::hos::hdomainobj::HDomainObj;
use crate::hos::hdomainsession::HDomainSession;
use crate::modules::fsp::fsp_init;

static mut IPC_MODULE_HANDLERS: BTreeMap<String, HClientSessionHandler> = BTreeMap::new();

pub fn ipc_init()
{
    fsp_init();
}

pub fn ipc_register_handler(service_name: String, handler: HClientSessionHandler)
{
    unsafe
    {
        IPC_MODULE_HANDLERS.insert(service_name, handler);
    }
}

pub fn ipc_get_handler(service_name: String) -> Option<HClientSessionHandler>
{
    unsafe
    {
        if let Some(handler) = IPC_MODULE_HANDLERS.get(&service_name) {
            return Some(*handler);
        }
        return None;
    }
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
            
            if let Some(handler) = ipc_get_handler(name) {
                /*
                if resp.hook_first_handle(handle, handler) {
                    //println_core!("sm::GetServiceHandle(`{}`) -> {:x}", name, handle);
                }
                */
                if let Some(handle) = resp.get_handle(0) {
                    //println_core!("sm::GetServiceHandle(`{}`) -> {:x}", name, handle);
                    
                    // TODO: Copied handles may not actually belong to parent
                    let mut service_hsession = sm_hsession.lock().new_from_parent();
                    
                    // Set handler
                    service_hsession.set_handler(handler);
                    
                    // Link new HClientSession to HOS handle
                    hipc_register_handle_clientsession(handle, Arc::new(Mutex::new(service_hsession)));
                }
            }
            
            return post_ctx;
        },
        2 => // RegisterService
        { 
            let name = pkt.read_str(0);
            //println_core!("sm::RegisterService(`{}`) from `{}`", name, vsvc_get_curpid_name());
        },
        3 => // UnregisterService
        { 
            let name = pkt.read_str(0);
            //println_core!("sm::UnregisterService(`{}`) from `{}`", name, vsvc_get_curpid_name());
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

pub async fn ipc_handle_syncrequest_control(mut pre_ctx: [u64; 32]) -> [u64; 32]
{
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let pkt = hipc_get_packet();

    match pkt.get_cmd_id()
    {
        0 => // ConvertCurrentObjectToDomain
        {
            if let Some(mut hsession) = hipc_get_handle_clientsession(handle)
            {
                // Wait for SVC to complete
                let post_ctx = SvcWait::new(pre_ctx).await;
                let resp = hipc_get_packet();
                
                // Domain obj out
                let obj = resp.read_u32(0);
            
                let hsession_locked = hsession.lock();
                let conv = hsession_locked.convert_to_domain(handle, obj);
                
                // Registration pair
                let domain_obj = conv.0;
                let domain_sess = conv.1;
                
                hipc_register_domain(domain_obj, Arc::new(Mutex::new(domain_sess)));
                return post_ctx;
            }
        },
        1 => // CopyFromCurrentDomain
        {
            let obj = pkt.read_u32(0);
            let mut handler_opt: Option<HClientSessionHandler> = None;
            if let Some(mut hsession) = hipc_get_domain_session(HDomainObj::from_curpid(handle, obj))
            {
                let hsession_locked = hsession.lock();

                handler_opt = hsession_locked.get_handler();
            }

            // If there's a handler, copy it to new session handle
            if let Some(handler) = handler_opt
            {
                // Wait for SVC to complete
                let post_ctx = SvcWait::new(pre_ctx).await;
                let resp = hipc_get_packet();

                // Get native handle
                let session_handle = resp.read_u32(0);
                if let Some(mut hsession) = hipc_get_handle_clientsession(handle)
                {
                    let mut service_hsession = hsession.lock().new_from_parent();
                
                    service_hsession.set_handler(handler);
                
                    // Link new HClientSession to HOS handle
                    hipc_register_handle_clientsession(session_handle, Arc::new(Mutex::new(service_hsession)));
                }
                return post_ctx;
            }
            return pre_ctx;
            //println_core!("CopyFromCurrentDomain");
        },
        2 | 4 => // CloneCurrentObject, CloneCurrentObjectEx
        {
            // Wait for SVC to complete
            let post_ctx = SvcWait::new(pre_ctx).await;
            let resp = hipc_get_packet();

            // Get native handle
            let session_handle = resp.read_u32(0);
            if let Some(mut hsession) = hipc_get_handle_clientsession(handle)
            {
                let mut service_hsession = hsession.lock().new_from_parent();
            
                if let Some(handler) = hsession.lock().get_handler() {
                    service_hsession.set_handler(handler);
                }
            
                // Link new HClientSession to HOS handle
                hipc_register_handle_clientsession(session_handle, Arc::new(Mutex::new(service_hsession)));
            }
            return post_ctx;
            //println_core!("CloneCurrentObject");
        },
        3 => // QueryPointerBufferSize
        {
            //println_core!("QueryPointerBufferSize");
        },
        _ => {}
    }
    return pre_ctx;
}

pub async fn ipc_handle_syncrequest(mut pre_ctx: [u64; 32]) -> [u64; 32]
{
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
        
    let pkt = hipc_get_packet();
    match pkt.get_type()
    {
        PKT_TYPE_LEGACYREQEST | PKT_TYPE_REQUEST | PKT_TYPE_REQUESTWITHCONTEXT =>
        {
            if pkt.is_domain()
            {
                let obj = pkt.get_domain_id();

                match pkt.get_domain_cmd()
                {
                    1 => // Request
                    {
                        let mut handler_opt: Option<HClientSessionHandler> = None;
                        if let Some(mut hsession) = hipc_get_domain_session(HDomainObj::from_curpid(handle, obj))
                        {
                            let hsession_locked = hsession.lock();

                            handler_opt = hsession_locked.get_handler();
                        }

                        // If there's a handler, let it take over
                        if let Some(handler) = handler_opt
                        {
                            return handler(pre_ctx).await;
                        }
                    },
                    2 => // Delete
                    {
                        hipc_remove_domain(HDomainObj::from_curpid(handle, obj));
                        return pre_ctx;
                    },
                    _ => { return pre_ctx; }
                }
            }
            else
            {
                // Get port struct
                let mut handler_opt: Option<HClientSessionHandler> = None;
                if let Some(mut hsession) = hipc_get_handle_clientsession(handle)
                {
                    let hsession_locked = hsession.lock();

                    //println_core!("svcSendSyncRequest from `{}` to handle {:x}", vsvc_get_curpid_name(), handle);
                    //println!("          `{}` -> `{}`", vsvc_get_curpid_name(), vsvc_get_pid_name(hsession_locked.parent_port_pid as u32));
                    
                    handler_opt = hsession_locked.get_handler();
                }
                
                // If there's a handler, let it take over
                if let Some(handler) = handler_opt
                {
                    return handler(pre_ctx).await;
                }
            }
        },
        PKT_TYPE_CLOSE =>
        {
            if pkt.is_domain()
            {
                //TODO?
            }
            hipc_close_handle(handle);
        }
        PKT_TYPE_LEGACYCONTROL | PKT_TYPE_CONTROL | PKT_TYPE_CONTROLWITHCONTEXT =>
        {
            return ipc_handle_syncrequest_control(pre_ctx).await;
        }
        _ => { return pre_ctx; }
    }
    

    //panic!("asdf");
    
    return pre_ctx;
}
