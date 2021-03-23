/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::ptr::NonNull;
use crate::io::uart::*;
use crate::io::timer::*;
use crate::usbd::usbd::*;
use crate::usbd::cdc::*;
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

static LOGGER_MUTEX: spin::Mutex<()> = spin::Mutex::new(());

static mut LOGGER_DATA: [Option<VecDeque<u8>>; 8] = [None, None, None, None, None, None, None, None];

#[macro_use]
mod logger {
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
        ($fmt:expr) => { crate::logger::logln_unsafe($fmt); };
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
        for i in 0..8
        {
            LOGGER_DATA[i] = Some(VecDeque::new());
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
        LOGGER_MUTEX.force_unlock();
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

    cdc_send(usbd, data.as_bytes(), data.len());
}

pub fn log_usb_raw(data: &[u8])
{
    let usbd = get_usbd();

    cdc_send(usbd, data, data.len());
}

pub fn log_process()
{
    unsafe
    {
        critical_start();
        
        // Process data later if this gets called mid-log
        // TODO per-core mutex?
        // TODO timestamps?
        let lock_try = LOGGER_MUTEX.try_lock();
        if (!lock_try.is_some())
        {
            critical_end();
            return;
        }
        let lock = lock_try.unwrap();
        
        loop
        {
            for core_iter in 0..8
            {
                let mut logger_data = LOGGER_DATA[core_iter].as_mut().unwrap();
                
                if (logger_data.is_empty())
                {
                    continue;
                }

                let data = logger_data.make_contiguous();

                let mut next_line = data.len();
                for i in 0..data.len()
                {
                    if data[i] == '\n' as u8 {
                        next_line = i+1;
                        break;
                    }
                }
                
                log_uarta_raw(&data[0..next_line]);
                log_usb_raw(&data[0..next_line]);
                
                let data_len = data.len();
                drop(data);
                
                if next_line >= data_len {
                    logger_data.clear()
                }
                else {
                    let mut i = 0;
                    logger_data.retain(|_| (i >= next_line, i += 1).0);
                }
            }
            
            let mut is_done = true;
            for core_iter in 0..8
            {
                let mut logger_data = LOGGER_DATA[core_iter].as_mut().unwrap();
                
                if (!logger_data.is_empty())
                {
                    is_done = false;
                }
            }
            
            if is_done { break; }
        }
        
        critical_end();
    }
}

pub fn log(data: &str)
{
    unsafe
    {
        critical_start();
        
        let lock = LOGGER_MUTEX.lock();

        let mut logger_data = LOGGER_DATA[get_core() as usize].as_mut().unwrap();
        
        for byte in data.bytes()
        {
            logger_data.push_back(byte);
        }
        
        critical_end();
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

pub fn log_raw(data: &[u8])
{
    log_uarta_raw(data);
    log_usb_raw(data);
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
