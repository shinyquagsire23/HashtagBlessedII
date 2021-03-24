/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::usbd::usbd::*;
use core::mem;
use core::str;
use crate::arm::threading::*;
use alloc::vec::Vec;
use crate::logger::*;
use crate::util::t210_reset;
use alloc::string::String;
use crate::io::timer::*;

pub const DEBUG_BULK_PKT_SIZE: u16 = (64);

pub struct DebugGadget
{
    isactive: bool,
    enabled: bool,
    has_acked: bool,
    if0: u8,
    if0_epBulkOut: u8,
    if0_epBulkIn: u8,
    cmd_buf: String,
    
}

impl DebugGadget
{
    pub const fn empty() -> Self
    {
        DebugGadget
        {
            isactive: false,
            enabled: false,
            has_acked: false,
            if0: 0xff,
            if0_epBulkOut: 0xff,
            if0_epBulkIn: 0xff,
            cmd_buf: String::new(),
        }
    }
}

static mut DEBUG_INST: DebugGadget = DebugGadget::empty();

pub fn debug_process_cmd()
{
    let debug = get_debug();
    
    if (debug.cmd_buf == "rcm")
    {
        unsafe {t210_reset();}
        loop {}
    }
    else if (debug.cmd_buf == "irqshow")
    {
        //irq_show();
    }
    else
    {
        println!("> Unknown command `{}`", debug.cmd_buf);
    }
    
    debug.cmd_buf.clear();
}

pub fn debug_disable()
{
    let debug = get_debug();
    debug.enabled = false;
}

pub fn debug_enable()
{
    let debug = get_debug();
    debug.enabled = true;
}

pub fn debug_active() -> bool
{
    let debug = get_debug();
    
    // keep compiler from optimizing this in a dumb way
    unsafe { asm!("add xzr, xzr, {0}", in(reg) &debug); }
    
    return debug.isactive;
}

pub fn debug_acked() -> bool
{
    let debug = get_debug();
    
    // keep compiler from optimizing this in a dumb way
    unsafe { asm!("add xzr, xzr, {0}", in(reg) &debug); }
    
    return debug.has_acked;
}

pub fn debug_send(usbd: &mut UsbDevice, data: &[u8])
{
    let debug = get_debug();
    
    if (!debug.isactive) { return; }

    let is_enabled = /*debug.enabled && debug.isactive &&*/ (get_core() == 0);

    if (data.len() == 0)
    {
        //mutexUnlock(&debug_send_mutex);
        return;
    }

    if (is_enabled /*&& mutexTryLock(&debug_usb_mutex)*/)
    {
        let mut bytes_to_send: i32 = (data.len() & 0x7FFFFFFF) as i32;
        //mutexUnlock(&debug_send_mutex);

        let mut i = 0;
        loop
        {
            if (bytes_to_send <= 0) { break; }
            
            let mut to_send: usize = bytes_to_send as usize;
            if (to_send > 64) {
                to_send = 64;
            }
            for j in 0..10
            {
                if(usbd.ep_tx(debug.if0_epBulkIn, to_u64ptr!(&data[i]), to_send, true) == UsbdError::Success) {
                    break;
                }
                timer_wait(1000);
            }
            bytes_to_send -= to_send as i32;
            i += to_send;
        }
        //mutexUnlock(&debug_usb_mutex);

        return;
    }
    //mutexUnlock(&debug_send_mutex);
}

pub fn debug_if0_recvcomplete(usbd: &mut UsbDevice, epNum: u8)
{
    unsafe
    {
    let debug = get_debug();

    let p_pkt_data = usbd.get_xferbuf(debug.if0_epBulkOut);
    let pkt_data: *mut u8 = p_pkt_data as _;
    let len = usbd.get_bytes_received(debug.if0_epBulkOut);
    
    // Magic value to start debugging
    if len >= 4 && !debug.has_acked {
        let pkt_magic: *mut u32 = p_pkt_data as _;
        if (pkt_magic.read() == 0xF00FF00F) {
            debug.has_acked = true;
            return;
        }
    }
    
    // Parse binary command
    if pkt_data.read() == 1 {
        
        return;
    }
    
    //println!("read {} bytes", len);
    
    let mut to_send: Vec<u8> = Vec::with_capacity(DEBUG_BULK_PKT_SIZE as usize);
    let p_to_send = to_u64ptr!(to_send.as_mut_ptr());
    
    // Send our data
    log_raw(&to_send.as_slice());
    
    // Convert the strings or whatever
    for i in 0..(len as usize)
    {
        let val = pkt_data.offset(i as isize).read();
        if (val == 0) { continue; }

        debug.cmd_buf.push(val as char);
        to_send.push(val);
        
        if (val == '\n' as u8)
        {
            // Send our data
            log_raw(&to_send.as_slice());
    
            debug.cmd_buf.pop();
            debug_process_cmd();
            to_send.clear();
        }
    }
    
    // Send our data
    log_raw(&to_send.as_slice());
    }
}

pub fn debug_if0_sendcomplete(usbd: &mut UsbDevice, epNum: u8)
{
    let debug = get_debug();
}

pub fn debug_if0_recvfail(usbd: &mut UsbDevice, epNum: u8)
{
    let debug = get_debug();

    let len = usbd.get_bytes_received(debug.if0_epBulkOut);
    
    //println!("read {} bytes then failed!", len);
}

pub fn debug_if0_sendfail(usbd: &mut UsbDevice, epNum: u8)
{
    let debug = get_debug();
    
    let len = usbd.get_bytes_received(debug.if0_epBulkIn);
    
    //println!("sent {} bytes then failed!", len);
}

pub fn debug_if0_recvidle(usbd: &mut UsbDevice, epNum: u8)
{
    let debug = get_debug();
    
    debug.enabled = true;
    debug.isactive = true;

    // Get more data
    usbd.ep_txfer_start(debug.if0_epBulkOut, DEBUG_BULK_PKT_SIZE as usize, false);
}

pub fn debug_if0_sendidle(usbd: &mut UsbDevice, epNum: u8)
{
    let debug = get_debug();
    
    debug.enabled = true;
    debug.isactive = true;
}

pub fn debug_reset_hook(usbd: &mut UsbDevice)
{
    let debug = get_debug();
    
    debug.isactive = false;
    debug.enabled = false;
    debug.has_acked = false;
    debug.cmd_buf.clear();
}

pub fn get_debug() -> &'static mut DebugGadget
{
    unsafe
    {
        &mut DEBUG_INST
    }
}

pub fn debug_init()
{
    let usbd = get_usbd();
    let debug = get_debug();
    
    // We allocate two interfaces, one has an interrupt EP (unused?) 
    // and the other has two bulk endpoints for each direction
    debug.if0 = usbd.create_interface(2);
    
    // We associate the former interface w/ the latter
    // (adds a device descriptor associating the two)
    usbd.get_interface(debug.if0).associatedNum = 2;

    // Interface0 info
    usbd.get_interface(debug.if0).class = 0xFF;
    usbd.get_interface(debug.if0).subclass = 0xFF;
    usbd.get_interface(debug.if0).protocol = 0xFF;
    
    // Set up if0 endpoints
    debug.if0_epBulkOut = usbd.get_interface(debug.if0).endpointStart+0;
    debug.if0_epBulkIn = usbd.get_interface(debug.if0).endpointStart+1;
    usbd.get_ep(debug.if0_epBulkOut).ep_construct(DEBUG_BULK_PKT_SIZE, USB_EPATTR_TTYPE_BULK, 0);
    usbd.get_ep(debug.if0_epBulkIn).ep_construct(DEBUG_BULK_PKT_SIZE, USB_EPATTR_TTYPE_BULK, 0);
    
    // Register all of our handlers
    usbd.register_complete_handler(debug.if0_epBulkOut, debug_if0_recvcomplete);
    usbd.register_complete_handler(debug.if0_epBulkIn, debug_if0_sendcomplete);
    usbd.register_fail_handler(debug.if0_epBulkOut, debug_if0_recvfail);
    usbd.register_fail_handler(debug.if0_epBulkIn, debug_if0_sendfail);
    usbd.register_idle_handler(debug.if0_epBulkOut, debug_if0_recvidle);
    usbd.register_idle_handler(debug.if0_epBulkIn, debug_if0_sendidle);

    usbd.register_reset_hook(debug_reset_hook);
    
    debug_reset_hook(usbd);
}

pub fn debug_fini()
{
    let usbd = get_usbd();
    let debug = get_debug();
    
    usbd.remove_complete_handler(debug.if0_epBulkOut);
    usbd.remove_complete_handler(debug.if0_epBulkIn);
}
