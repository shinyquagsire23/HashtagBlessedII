/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::ptr::NonNull;
use crate::io::uart::*;

pub fn logger_init()
{
    
}

pub fn log(data: &str)
{
    let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
    uart_a.writeStr(data);
    //uart_a.waitForWrite();
}

pub fn logu32(data: u32)
{
    let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
    let mut data_shift: u32 = data;
    
    for i in 0..8
    {
        let nibble: u8 = ((data_shift & 0xF0000000) >> 28) as u8;
        
        let nibble_str = match nibble
        {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "a",
            11 => "b",
            12 => "c",
            13 => "d",
            14 => "e",
            15 => "f",
            _ => "",
        };
        
        uart_a.writeStr(nibble_str);
        
        data_shift <<= 4;
    }
    
    uart_a.writeStr("\n\r");
    
    //uart_a.waitForWrite();
}

pub fn logu16(data: u16)
{
    let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
    let mut data_shift: u16 = data;
    
    for i in 0..4
    {
        let nibble: u8 = ((data_shift & 0xF000) >> 12) as u8;
        
        let nibble_str = match nibble
        {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "a",
            11 => "b",
            12 => "c",
            13 => "d",
            14 => "e",
            15 => "f",
            _ => "",
        };
        
        uart_a.writeStr(nibble_str);
        
        data_shift <<= 4;
    }
    
    uart_a.writeStr("\n\r");
    
    //uart_a.waitForWrite();
}

pub fn logu8(data: u8)
{
    let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
    let mut data_shift: u8 = data;
    
    for i in 0..2
    {
        let nibble: u8 = ((data_shift & 0xF0) >> 4) as u8;
        
        let nibble_str = match nibble
        {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "a",
            11 => "b",
            12 => "c",
            13 => "d",
            14 => "e",
            15 => "f",
            _ => "",
        };
        
        uart_a.writeStr(nibble_str);
        
        data_shift <<= 4;
    }
    
    uart_a.writeStr("\n\r");
    
    //uart_a.waitForWrite();
}

#[defmt::global_logger]
struct Logger;

impl defmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        let mut uart_a: UARTDevice = UARTDevice::new(UARTDevicePort::UartA);
        uart_a.writeBytes(bytes);
    }
}

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<NonNull<dyn defmt::Write>> {
            Some(NonNull::from(&Logger as &dyn defmt::Write))
    }

    unsafe fn release(_: NonNull<dyn defmt::Write>) {
    }
}

#[macro_use]
mod logger {
    macro_rules! logasdf {
        ($a:expr) => {
            
        }
    }
}
