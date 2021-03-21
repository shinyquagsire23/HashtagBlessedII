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
}

pub fn logger_init()
{
    
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

pub fn log(data: &str)
{
    log_uarta(data);
    log_usb(data);
}

pub fn log_raw(data: &[u8])
{
    log_uarta_raw(data);
    log_usb_raw(data);
}

pub fn logln(data: &str)
{
    log(data);
    log("\n\r");
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
