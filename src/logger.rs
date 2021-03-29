/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::ptr::NonNull;
use crate::io::uart::*;
use crate::io::timer::*;
use crate::usbd::usbd::*;
use crate::usbd::debug::*;
use core::str;
use spin::Mutex;
use alloc::sync::Arc;
use alloc::collections::vec_deque::VecDeque;
use crate::vm::virq::*;
use crate::task::*;
use crate::task::sleep::*;
use crate::arm::ticks::*;
use crate::util::*;
use crate::arm::threading::*;

static LOGGER_MUTEX: [spin::Mutex<()>; 8] = [spin::Mutex::new(()), spin::Mutex::new(()), spin::Mutex::new(()), spin::Mutex::new(()), spin::Mutex::new(()), spin::Mutex::new(()), spin::Mutex::new(()), spin::Mutex::new(())];

static mut LOGGER_DATA_COMB: spin::Mutex<Option<VecDeque<u8>>> = spin::Mutex::new(None);
static mut LOGGER_DATA: [Option<VecDeque<u8>>; 8] = [None, None, None, None, None, None, None, None];
static mut LOGGER_CMD: [Option<VecDeque<u8>>; 8] = [None, None, None, None, None, None, None, None];

#[macro_use]
mod logger {
    macro_rules! println_core {
        () => { };
        ($fmt:expr) => {
            crate::logger::log(&format!("(core {}) ", crate::arm::threading::get_core()));
            crate::logger::logln($fmt);
        };
        ($fmt:expr, $($arg:tt)*) => {{
            let text = format!($fmt, $($arg)*);
            crate::logger::logln(&text);
        }};
    }
    
    macro_rules! println {
        () => { };
        ($fmt:expr) => { crate::logger::logln($fmt); };
        ($fmt:expr, $($arg:tt)*) => {{
            let text = format!($fmt, $($arg)*);
            crate::logger::logln(&text);
        }};
    }
    
    macro_rules! print {
        () => { };
        ($fmt:expr) => { crate::logger::log($fmt); };
        ($fmt:expr, $($arg:tt)*) => {{
            let text = format!($fmt, $($arg)*);
            crate::logger::log(&text);
        }};
    }
    
    macro_rules! println_unsafe {
        () => { };
        ($fmt:expr) => { crate::logger::logln_unsafe($fmt); };
        ($fmt:expr, $($arg:tt)*) => {{
            let text = format!($fmt, $($arg)*);
            crate::logger::logln_unsafe(&text);
        }};
    }
    
    macro_rules! println_uarta {
        () => { };
        ($fmt:expr) => { crate::logger::log_uarta($fmt); crate::logger::log_uarta("\r\n"); };
        ($fmt:expr, $($arg:tt)*) => {{
            let text = format!($fmt, $($arg)*);
            crate::logger::log_uarta(&text);
            crate::logger::log_uarta("\r\n");
        }};
    }
    
    macro_rules! print_unsafe {
        () => { };
        ($fmt:expr) => { crate::logger::log_unsafe($fmt); };
        ($fmt:expr, $($arg:tt)*) => {{
            let text = format!($fmt, $($arg)*);
            crate::logger::log_unsafe(&text);
        }};
    }
}

pub fn logger_init()
{
    unsafe
    {
        LOGGER_DATA_COMB = spin::Mutex::new(Some(VecDeque::new()));
        for i in 0..8
        {
            LOGGER_DATA[i] = Some(VecDeque::new());
            LOGGER_CMD[i] = Some(VecDeque::new());
        }
    }
    
    task_run(logger_task());
}

pub async fn logger_task()
{
    loop
    {
        log_process();
        SleepNs::new(ms_to_ns(1)).await;
    }
}

pub fn logger_unsafe_override()
{
    unsafe
    {
        for i in 0..8
        {
            LOGGER_MUTEX[i].force_unlock();
        }
        LOGGER_DATA_COMB.force_unlock();
    }
}

pub fn log_uarta(data: &str)
{
    let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
    uart_a.write_str(data);
    //uart_a.wait_for_write();
}

pub fn log_uarta_raw(data: &[u8])
{
    let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
    uart_a.write_bytes(data);
    //uart_a.wait_for_write();
}

pub fn log_usb(data: &str)
{
    let usbd = get_usbd();
    
    debug_send(usbd, data.as_bytes());
}

pub fn log_usb_raw(data: &[u8])
{
    let usbd = get_usbd();

    debug_send(usbd, data);
}

pub fn logger_clear_unprocessed()
{
    unsafe
    {
        let irq_lock = critical_start();
        
        {
            LOGGER_DATA_COMB.lock().as_mut().unwrap().clear();
        }

        for core_iter in 0..8
        {
            let lock = LOGGER_MUTEX[core_iter].lock();

            let mut logger_cmd = LOGGER_CMD[core_iter].as_mut().unwrap();

            logger_cmd.clear();
        }
        
        for core_iter in 0..8
        {
            let lock = LOGGER_MUTEX[core_iter].lock();

            let mut logger_cmd = LOGGER_DATA[core_iter].as_mut().unwrap();

            logger_cmd.clear();
        }
        
        critical_end(irq_lock);
    }
}

pub fn log_process_cmd()
{
    unsafe
    {
        let irq_lock = critical_start();

        let mut logger_cmd_copy: [Option<VecDeque<u8>>; 8] = [None, None, None, None, None, None, None, None];
        
        for core_iter in 0..8
        {
            let lock = LOGGER_MUTEX[core_iter].lock();

            let mut logger_cmd = LOGGER_CMD[core_iter].as_mut().unwrap();

            logger_cmd_copy[core_iter] = Some(logger_cmd.split_off(0));
        }
        
        // USB side-channel data
        loop
        {
            for core_iter in 0..8
            {
                let mut logger_data = logger_cmd_copy[core_iter].as_mut().unwrap();
                
                if (logger_data.is_empty())
                {
                    continue;
                }

                log_usb_raw(logger_data.make_contiguous());
                
                logger_data.clear();
            }
            
            let mut is_done = true;
            for core_iter in 0..8
            {
                let mut logger_data = logger_cmd_copy[core_iter].as_mut().unwrap();
                
                if (!logger_data.is_empty())
                {
                    is_done = false;
                }
            }
            
            if is_done { break; }
        }
        
        critical_end(irq_lock);
    }
}

pub fn log_process()
{
    unsafe
    {
        let irq_lock = critical_start();

        // Main log
        let usbd = get_usbd();
        
        // About 46ns per character?
        
        {
            let mut lock_comb = LOGGER_DATA_COMB.try_lock();
            let mut lines_flushed = 0;
            
            if let Some(mut comb) = lock_comb {
                let logger_data = comb.as_mut().unwrap();
                for i in 0..256
                {
                    match logger_data.pop_front() {
                        Some(data) => {
                            //log_uarta_raw(data);
                            debug_send_byte(usbd, data);
                            if data == '\n' as u8 {
                                debug_flush(usbd);
                                lines_flushed += 1;
                                if lines_flushed > 3 {
                                    break;
                                }
                            }
                        },
                        None => { break; }
                    }
                }
            }
        }
        
        //log_process_cmd();
        
        debug_flush(usbd);
        
        critical_end(irq_lock);
    }
}

pub fn log_try_flush(core: u8, flush_remainder: bool)
{
    unsafe
    {
        let irq_lock = critical_start();
        let lock = LOGGER_MUTEX[core as usize].lock();

        let mut logger_data = LOGGER_DATA[core as usize].as_mut().unwrap();
        
        let mut i = 0;
        loop
        {
            if i >= logger_data.len() { break; }
            
            // Handle any blinking/spinning/etc
            if logger_data[i] == '\r' as u8 {
            
                let mut to_split = i+1;
                if i < logger_data.len()-1 && logger_data[i+1] == '\n' as u8 {
                    to_split += 1
                }
                // Attempt to lock, if busy just handle later.
                let mut lock_comb = LOGGER_DATA_COMB.try_lock();
                if let Some(mut comb) = lock_comb {
                    let mut split_remainder = logger_data.split_off(to_split);
                    comb.as_mut().unwrap().append(&mut logger_data);
                    *logger_data = split_remainder;
                }
                
                i = 0;
                continue;
            }
            // Otherwise, find newline to flush data to
            else if logger_data[i] == '\n' as u8 {
            
                // Attempt to lock, if busy just handle later.
                let mut lock_comb = LOGGER_DATA_COMB.try_lock();
                if let Some(mut comb) = lock_comb {
                    let mut split_remainder = logger_data.split_off(i+1);
                    comb.as_mut().unwrap().append(&mut logger_data);
                    *logger_data = split_remainder;
                }
                
                i = 0;
                continue;
            }
            
            i += 1;
        }
        
        if flush_remainder {
            let mut split = logger_data.split_off(0);
            LOGGER_DATA_COMB.lock().as_mut().unwrap().append(&mut split);
        }
        
        critical_end(irq_lock);
    }
}

pub fn log_try_flush_all()
{
    for i in 0..8
    {
        log_try_flush(i, false);
    }
}

pub fn log(data: &str)
{
    log_raw(data.as_bytes());
}

pub fn log_raw(data: &[u8])
{
    unsafe
    {
        let irq_lock = critical_start();
        
        {
            let lock = LOGGER_MUTEX[get_core() as usize].lock();
            let mut logger_data = LOGGER_DATA[get_core() as usize].as_mut().unwrap();
            
            for byte in data
            {
                logger_data.push_back(*byte);
            }
        }
        log_try_flush(get_core(), false);

        critical_end(irq_lock);
    }
}

pub fn log_cmd(data: &[u8])
{
    unsafe
    {
        let irq_lock = critical_start();
        
        let lock = LOGGER_MUTEX[get_core() as usize].lock();

        let mut logger_data = LOGGER_CMD[get_core() as usize].as_mut().unwrap();
        
        for byte in data
        {
            logger_data.push_back(*byte);
        }
        
        critical_end(irq_lock);
    }
}

pub fn log_unsafe(data: &str)
{
    log_uarta_raw(data.as_bytes());
    log_usb_raw(data.as_bytes());
}

pub fn logln_unsafe(data: &str)
{
    log_unsafe(data);
    log_unsafe("\r\n");
}

pub fn logln(data: &str)
{
    log(data);
    log("\r\n");
}

pub fn logu32(data: u32)
{
    println!("{:08x}", data);
}

pub fn logu16(data: u16)
{
    println!("{:04x}", data);
}

pub fn logu8(data: u8)
{
    println!("{:02x}", data);
}

pub fn hexdump(prefix: &str, addr: u64, len: usize)
{
    println!("{}:", prefix);
    for i in 0..len
    {
        let byte = peek8(addr + i as u64);
        
        if (i != 0 && (i % 16) == 0)
        {
            println!("");
        }
        
        print!(" {:02x}", byte);
    }
    println!("");
}
