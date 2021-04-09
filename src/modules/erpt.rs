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

pub fn erpt_init()
{
    ipc_register_handler(String::from("erpt:c"), handle_erpt_boxed);
}

/*
NumericU64, 0
NumericU32, 1
NumericI64, 2
NumericI32, 3
String,     4
U8Array,    5
U32Array,   6
U64Array,   7
I32Array,   8
I64Array,   9
Bool,       10
NumericU16, 11
NumericU8,  12
NumericI16, 13
NumericI8,  14
I8Array,    15
*/

async fn handle_erpt(mut pre_ctx: [u64; 32], hobj: HObject) -> [u64; 32]
{
    let pkt = hipc_get_packet();
    
    let handle = (pre_ctx[0] & 0xFFFFFFFF) as u32;
    let hsession = hipc_get_handle_clientsession(handle).unwrap();
    
    match pkt.get_cmd_id()
    {
        0 => // SubmitContext
        {
            //println_core!("erpt::SubmitContext(...) from `{}`", vsvc_get_curpid_name());

            return pre_ctx;
        },
        1 => // CreateReportV0
        {
            let desc_ctx_entry = pkt.get_send(0).unwrap();
            let desc_reportlist = pkt.get_send(1).unwrap();
            let desc_reportmetadata = pkt.get_send(2).unwrap();
            let report_type = pkt.read_u32(0);
            
            let field_cnt = peek32(desc_ctx_entry.get_addr_el2() + 4);
            let category_id = peek32(desc_ctx_entry.get_addr_el2() + 8);
            let field_list_ptr = peek64(desc_ctx_entry.get_addr_el2() + 0x150);
            let field_free_cnt = peek32(desc_ctx_entry.get_addr_el2() + 0x158);
            let field_size = peek32(desc_ctx_entry.get_addr_el2() + 0x15c);
            
            for i in 0..(field_cnt as u64)
            {
                let field_id = peek32(desc_ctx_entry.get_addr_el2() + 0x10 + (i*16) + 0);
                let field_type = peek32(desc_ctx_entry.get_addr_el2() + 0x10 + (i*16) + 4);
                let field_val = peek64(desc_ctx_entry.get_addr_el2() + 0x10 + (i*16) + 8);
                println_core!("entry: {:08x} {:08x} {:016x}", field_id, field_type, field_val);
                if field_type == 4 { // string
                    let idx = (field_val & 0xFFFFFFFF) as usize;
                    let val_size = ((field_val >> 32) & 0xFFFFFFFF) as usize;
                    let val = desc_reportlist.read_str(idx);
                    println_core!("  -> `{}`", val);
                }
                else if field_type == 0 { // u64
                    println_core!("  -> {:016x}", field_val);
                }
                else if field_type == 1 { // u32
                    println_core!("  -> {:08x}", field_val);
                }
                else if field_type == 2 { // i64
                    println_core!("  -> {:016x}", field_val);
                }
                else if field_type == 3 { // i32
                    println_core!("  -> {:08x}", field_val);
                }
                else if field_type == 10 { // bool
                    println_core!("  -> {}", (field_val & 0xFF) != 0);
                }
            }
            
            for i in 0..((desc_reportlist.size as u64) / 8)
            {
                let val = peek64(desc_reportlist.get_addr_el2() + i*8);
                //println_core!("list {:08x}: {:016x}", i*8, val);
            }
            
            for i in 0..((desc_reportmetadata.size as u64) / 8)
            {
                let val = peek64(desc_reportmetadata.get_addr_el2() + i*8);
                //println_core!("meta {:08x}: {:016x}", i*8, val);
            }
            
            println_core!("erpt::CreateReportV0({:x}, ...) from `{}`", report_type, vsvc_get_curpid_name());

            return pre_ctx;
        },
        _ => 
        {
            println_core!("erpt::Cmd{}(...) from `{}`", pkt.get_cmd_id(), vsvc_get_curpid_name());

            return pre_ctx; 
        }
    }

    return pre_ctx;
}

fn handle_erpt_boxed(mut pre_ctx: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>> {
    Box::pin(handle_erpt(pre_ctx, hobj))
}
