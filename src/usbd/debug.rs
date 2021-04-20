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
use spin::Mutex;
use alloc::collections::vec_deque::VecDeque;
use crate::vm::vsvc::*;
use crate::vm::vmmu::ipaddr_to_paddr;
use crate::util::peek64;

pub const TTB_ENTRY_ATTR_MASK: u64 = 0xFFF0000000000000;
pub const TTB_ENTRY_ATTR_SHIFT: usize = (52);
pub const TTB_ENTRY_LOWER_ATTR_MASK: u64 = 0x00000000000007FC;
pub const TTB_ENTRY_ADDR_MASK: u64 = 0x00007FFFFFFFF800;
pub const TTB_ENTRY_TYPE_MASK: u64 = 0x0000000000000003;

pub const DEBUG_BULK_PKT_SIZE: u16 = (64);

pub struct DebugGadget
{
    is_initted: bool,
    isactive: bool,
    enabled: bool,
    has_acked: bool,
    if0: u8,
    if0_epBulkOut: u8,
    if0_epBulkIn: u8,
    cmd_buf: spin::Mutex<String>,
    log_buf: spin::Mutex<Option<VecDeque<u8>>>,
    bincmd_buf: spin::Mutex<Option<VecDeque<u8>>>,
    bincmd_toread: u8,
}

impl DebugGadget
{
    pub const fn empty() -> Self
    {
        DebugGadget
        {
            is_initted: false,
            isactive: false,
            enabled: false,
            has_acked: false,
            if0: 0xff,
            if0_epBulkOut: 0xff,
            if0_epBulkIn: 0xff,
            cmd_buf: spin::Mutex::new(String::new()),
            log_buf: spin::Mutex::new(None),
            bincmd_buf: spin::Mutex::new(None),
            bincmd_toread: 0,
        }
    }
}

static mut DEBUG_INST: DebugGadget = DebugGadget::empty();

pub fn debug_get_cmd_buf() -> String
{
    let debug = get_debug();
    let lock = debug.cmd_buf.lock();

    return lock.clone();
}

fn debug_print_ttbr(addr: u64, vaddr_base: u64, level: u8) {
    if addr == 0 || level > 2 {
        return;
    }
    
    let mut last_upper_attr = 0;
    let mut last_lower_attr = 0;
    let mut last_vaddr = 0;
    let mut last_type = 0;
    let mut last_print = false;
    let mut has_printed = false;
    
    for i in 0..0x1000/8
    {
        let val = peek64(addr + (i*8));
        let val_next = if i == (0x1000/8)-1 { 0 } else { peek64(addr + ((i+1)*8)) };
        if val == 0 {
            continue;
        }
        
        let granularity = match level {
            0 => 0x40000000,
            1 => 0x200000,
            2 => 0x1000,
            _ => 0x0,
        };
        
        let vaddr = (vaddr_base + (i * granularity));// | 0xffffff8000000000; //TODO: 64-bit vaddrs?

        let val_addr = val & TTB_ENTRY_ADDR_MASK;
        let val_type = val & TTB_ENTRY_TYPE_MASK;
        let val_upper_attr = (val & TTB_ENTRY_ATTR_MASK) >> TTB_ENTRY_ATTR_SHIFT;
        let val_lower_attr = (val & TTB_ENTRY_LOWER_ATTR_MASK) >> 2;
        let val_ap = (val_lower_attr >> 4) & 3;
        let val_uxn = (val_upper_attr & 4) != 0;
        let val_pxn = (val_upper_attr & 2) != 0;
        
        let mut should_print = false;
        let is_table_ent = val_type == 3 && level < 2 && val_upper_attr == 0x80;
        
        if last_upper_attr != val_upper_attr
           || last_lower_attr != val_lower_attr
           || last_type != val_type
           || last_vaddr + granularity != vaddr
        {
            should_print = true;
        }
        
        last_type = val_type;
        last_upper_attr = val_upper_attr;
        last_lower_attr = val_lower_attr;
        last_vaddr = vaddr;
        
        if is_table_ent {
            should_print = true;
        }
        
        if !should_print && val_next == 0 {
            should_print = true;
        }
        
        if last_print != should_print && !should_print && has_printed {
            for i in 0..level {
                print!("  ");
            }
            println!("  ...");
        }
        
        last_print = should_print;
        
        if should_print {
            has_printed = true;
            for i in 0..level {
                print!("  ");
            }

            let uxn_char = if val_uxn { '-' } else { 'X' };
            let pxn_char = if val_pxn { '-' } else { 'X' };
            let mut mem_perms = format!("EL0 --{} EL1 --{}", uxn_char, pxn_char);
            match val_ap {
                0 => {
                    mem_perms = format!("EL0 --{} EL1 RW{}", uxn_char, pxn_char);
                },
                1 => {
                    mem_perms = format!("EL0 RW{} EL1 RW{}", uxn_char, pxn_char);
                },
                2 => {
                    mem_perms = format!("EL0 --{} EL1 R-{}", uxn_char, pxn_char);
                },
                3 => {
                    mem_perms = format!("EL0 R-{} EL1 R-{}", uxn_char, pxn_char);
                },
                _ => {},
            }

            match val_type {
                1 => // Block Entry
                {
                    println!("  {:016x} -> addr {:08x} {}", vaddr, val_addr, mem_perms);
                },
                3 => // Page Entry or Table Entry
                {
                    if is_table_ent {
                        println!("  {:016x} -> lv{} table addr {:08x}", vaddr, level + 1, val_addr);
                    }
                    else {
                        println!("  {:016x} -> addr {:08x} {}", vaddr, val_addr, mem_perms);
                    }
                },
                _ => {
                    println!("  {:016x} -> addr {:08x}, type {:x}, upper attr {:x}, lower_attr {:x}", vaddr, val_addr, val_type, val_upper_attr, val_lower_attr);
                }
            }
        }
        
        if is_table_ent {
            debug_print_ttbr(val_addr, vaddr, level + 1);
        }
        
    }
}

pub fn debug_process_cmd()
{
    let debug = get_debug();
    
    let command_full = &mut *debug.cmd_buf.lock();
    let mut command = command_full.clone();
    let mut args: Vec<String> = Vec::new();
    
    match command_full.split_once(' ') {
        Some(split) => {
            command = String::from(split.0);
            args = split.1.split_ascii_whitespace().map(|s| String::from(s)).collect();
        },
        None => {
        }
    };
    
    if (command == "rcm")
    {
        unsafe {t210_reset();}
        loop {}
    }
    else if (command == "irqshow")
    {
        //irq_show();
    }
    else if (command == "proc")
    {
        if (args.len() < 1)
        {
            println!("Usage: proc <operation>");
            println!("");
            println!("Valid operations:");
            println!(" - list: Lists all processes");
        }
        else
        {
            match args[0].as_str() {
                "list" => {
                    println!("Running Processes:");
                    for pid in vsvc_get_pid_list()
                    {
                        if pid == 0xFF { continue; }

                        println!("  {:3}: {}", pid, vsvc_get_pid_name(pid));
                    }
                    println!("");
                },
                _ => {
                    println!("Unknown operation `{}`", args[0]);
                }
            };
            
        }
    }
    else if (command == "ttbr")
    {
        if (args.len() < 1)
        {
            println!("Usage: ttbr <pid/name>");
        }
        else
        {
            match args[0].parse::<u32>() {
                Ok(pid) => {
                    let ttbr_addr = vsvc_get_pid_ttbr(pid);
                    let ttbr_translated = ipaddr_to_paddr(ttbr_addr);
                    println!("PID {} ({}) TTBR: {:016x} (->{:016x})", pid, vsvc_get_pid_name(pid), ttbr_addr, ttbr_translated);
                    debug_print_ttbr(ttbr_translated, 0, 0);
                }
                Error => {
                    let pid = vsvc_get_process_pid(&args[0]);
                    let ttbr_addr = vsvc_get_pid_ttbr(pid);
                    let ttbr_translated = ipaddr_to_paddr(ttbr_addr);
                    println!("PID {} ({}) TTBR: {:016x} (->{:016x})", pid, vsvc_get_pid_name(pid), ttbr_addr, ttbr_translated);
                    debug_print_ttbr(ttbr_translated, 0, 0);
                }
            };
            
        }
    }
    else if command == "help" || command == "?"
    {
        println!("Available Commands:");
        println!(" rcm - Reset to RCM mode");
        println!(" proc - Process commands");
        println!(" ttbr - Translation table register print");
        println!(" help, ? - Display help");
        println!("")
    }
    else if command == ""
    {
    }
    else
    {
        println!("> Unknown command `{}`", command);
    }
    
    command_full.clear();
}

pub fn debug_dispatch_bincmd()
{
    let debug = get_debug();    
    if (!debug.isactive) { return; }

    let mut lock = debug.bincmd_buf.lock();
    let mut bincmd_buf = lock.as_mut().unwrap();
    let bincmd_size = bincmd_buf.len();
    
    if bincmd_size <= 0 {
        return;
    }
    
    let bincmd_cmd = bincmd_buf[0];
    match bincmd_cmd {
        0 => {},
        1 => {
        }
        _ => {
            println_core!("debug: Received unknown debug cmd {:x}, pkt len {:x}", bincmd_cmd, bincmd_size);
        }
    }
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

fn debug_send_next(usbd: &mut UsbDevice)
{
    let debug = get_debug();
    
    if (!debug.isactive) { return; }
    
    let mut lock = debug.log_buf.lock();
    let mut log_buf = lock.as_mut().unwrap();
    
    if (log_buf.is_empty()) { return; }
    
    let mut to_send: usize = log_buf.len() as usize;
    
    if (to_send > 64) {
        to_send = 64;
    }
    
    // Command packet, read length
    if log_buf[0] == 1 && log_buf.len() >= 2 {
        to_send = (log_buf[1]+2) as usize;
    }
    else {
        // Keep command packets in their own individual bulk transfers
        // by truncating up to next command
        for i in 0..to_send
        {
            if log_buf[i] == 1 {
                to_send = i;
                break;
            }
        }
    }
    
    if to_send > log_buf.len() {
        to_send = log_buf.len();
    }
    
    if (to_send > 64) {
        to_send = 64;
    }
    
    let mut copied: [u8; 64] = [0; 64];
    for i in 0..to_send
    {
        copied[i] = log_buf[i];
    }

    usbd.ep_tx(debug.if0_epBulkIn, to_u64ptr!(&copied[0]), to_send, false);
}

pub fn debug_send_byte(usbd: &mut UsbDevice, data: u8)
{
    let debug = get_debug();
    
    if (!debug.isactive) { return; }
    
    // Copy data to our own outbuf
    {
        let mut lock = debug.log_buf.lock();
        let mut log_buf = lock.as_mut().unwrap();
        
        log_buf.push_back(data);
    }
}

pub fn debug_flush(usbd: &mut UsbDevice)
{
    let debug = get_debug();
    
    if (!debug.isactive) { return; }
    
    // Begin a transfer here if ep is already idle, next transfer begins on success/fail/idle
    if usbd.ep_status(debug.if0_epBulkIn) == UsbEpStatus::TxfrIdle
    {
        debug_send_next(usbd);
    }
}

pub fn debug_send(usbd: &mut UsbDevice, data: &[u8])
{
    let debug = get_debug();
    
    if (!debug.isactive) { return; }

    if (data.len() == 0)
    {
        return;
    }
    
    // Copy data to our own outbuf
    {
        let mut lock = debug.log_buf.lock();
        let mut log_buf = lock.as_mut().unwrap();
        
        for i in 0..data.len()
        {
            log_buf.push_back(data[i]);
        }
    }

    debug_flush(usbd);
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
    if pkt_data.read() == 1 && len >= 2 {
        let bincmd_len = pkt_data.offset(1).read();
        debug.bincmd_toread = bincmd_len;
        return;
    }
    
    // Convert the strings or whatever
    let mut is_escape = false;
    for i in 0..(len as usize)
    {
        let val = pkt_data.offset(i as isize).read();
        
        if debug.bincmd_toread > 0 {
            {
                let mut lock = debug.bincmd_buf.lock();
                let mut bincmd_buf = lock.as_mut().unwrap();
        
                bincmd_buf.push_back(val);
            }
            
            debug.bincmd_toread -= 1;
            if debug.bincmd_toread <= 0 {
                debug_dispatch_bincmd();
            }
            continue;
        }
        
        if (val == 0) { continue; }
        
        if (val == '\n' as u8)
        {
            // Send our data
            {
                let command = &mut *debug.cmd_buf.lock();
                println!("> {} ", command);
            }
            
            debug_process_cmd();
        }
        else if (val == 0xc4 || val == 0xc5)
        {
            is_escape = true;
            continue;
        }
        else if (is_escape && val == 0x87) // backspace
        {
            let command = &mut *debug.cmd_buf.lock();
            command.pop();
        }
        else if (is_escape && val == 0x84) // left
        {
        }
        else if (is_escape && val == 0x85) // right
        {
        }
        else if (is_escape && val == 0x83) // up
        {
        }
        else if (is_escape && val == 0x82) // down
        {
        }
        else if (is_escape && val >= 0x89 && val <= 0x94) // F1-F12
        {
        }
        else if (is_escape && val == 0x8b) // ins
        {
        }
        else if (is_escape && val == 0x8a) // del
        {
            
        }
        else
        {
            let command = &mut *debug.cmd_buf.lock();
            command.push(val as char);
        }
    }
    
    // Send our data
    //let command = &mut *debug.cmd_buf.lock();
    //print!("> {} \r", command);
    log_try_flush(get_core(), true);
    }
}

pub fn debug_if0_sendcomplete(usbd: &mut UsbDevice, epNum: u8)
{
    let debug = get_debug();
    
    let len = usbd.get_bytes_received(debug.if0_epBulkIn);
    
    {
        let mut lock = debug.log_buf.lock();
        let mut log_buf = lock.as_mut().unwrap();
        for i in 0..len
        {
            log_buf.pop_front();
        }
    }
    
    debug_send_next(usbd);
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
    
    {
        let mut lock = debug.log_buf.lock();
        let mut log_buf = lock.as_mut().unwrap();
        for i in 0..len
        {
            log_buf.pop_front();
        }
    }
    
    debug_send_next(usbd);
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
    
    debug_send_next(usbd);
}

pub fn debug_reset_hook(usbd: &mut UsbDevice)
{
    let debug = get_debug();
    
    debug.isactive = false;
    debug.enabled = false;
    debug.has_acked = false;
    
    let command = &mut *debug.cmd_buf.lock();
    command.clear();
    
    {
    let mut lock = debug.log_buf.lock();
    let mut log_buf = lock.as_mut().unwrap();
    log_buf.clear();
    }
    
    {
    let mut lock = debug.bincmd_buf.lock();
    let mut bincmd_buf = lock.as_mut().unwrap();
    bincmd_buf.clear();
    }
    debug.bincmd_toread = 0;
    
    logger_clear_unprocessed();
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

    if debug.is_initted
    {
        debug_reset_hook(usbd);
        return;
    }
    
    debug.is_initted = true;
    
    debug.log_buf = spin::Mutex::new(Some(VecDeque::new()));
    debug.cmd_buf = spin::Mutex::new(String::new());
    debug.bincmd_buf = spin::Mutex::new(Some(VecDeque::new()));
    debug.bincmd_toread = 0;

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
