/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![feature(assoc_char_funcs)]

extern crate rusb;
extern crate ncurses;

use std::thread;
use std::time;
use std::str;
use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use ncurses::*;
use binread::{BinRead, io::Cursor};

const VID_NINTENDO: u16 = 0x057e;
const PID_SWITCH: u16 = 0x2000;

#[derive(BinRead)]
#[br(magic = b"\x01")]
struct UsbCmdPacket {
    pkt_len: u8,

    #[br(little, count = pkt_len)]
    data: Vec<u8>,
}

#[macro_use]
macro_rules! println {
    () => { };
    ($fmt:expr) => { 
        addstr($fmt);
        addstr("\n");
        refresh();
    };
    ($fmt:expr, $($arg:tt)*) => {{
        let text = format!($fmt, $($arg)*);
        addstr(&text);
        addstr("\n");
        refresh();
    }};
}

macro_rules! print {
    () => { };
    ($fmt:expr) => { addstr($fmt); };
    ($fmt:expr, $($arg:tt)*) => {{
        let text = format!($fmt, $($arg)*);
        addstr(&text);
        refresh();
    }};
}

fn find_device() -> Option<(rusb::DeviceHandle<rusb::GlobalContext>, u8, u8, u8)>
{
    let mut handle: Option<rusb::DeviceHandle<rusb::GlobalContext>> = None;

    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();
        
        let vid = device_desc.vendor_id();
        let pid = device_desc.product_id();
        let version = device_desc.device_version();
        if vid != VID_NINTENDO || pid != PID_SWITCH || version != rusb::Version(1, 0, 1)
        {
            continue;
        }
        
        match device.open() {
               Err(_e) => { println!("{}", _e); continue;},
               Ok(h) => { handle = Some(h); break; },
        };
    }
    
    if !handle.is_some() {
        return None;
    }
    
    let handle_unwrap = handle.unwrap();
    let device = handle_unwrap.device();
    let device_desc = device.device_descriptor().unwrap();
    let num_configs = device_desc.num_configurations();
    let mut iface_num = 0xff;
    let mut ep_in_num = 0xff;
    let mut ep_out_num = 0xff;
    
    for config_idx in 0..num_configs
    {
        let config_desc = device.config_descriptor(config_idx).unwrap();
        for interface in config_desc.interfaces()
        {
            for iface_desc in interface.descriptors()
            {
                if iface_desc.class_code() != 0xFF
                    || iface_desc.sub_class_code() != 0xFF
                    || iface_desc.protocol_code() != 0xFF {
                    continue;
                }
                iface_num = interface.number();
                
                for endpoint in iface_desc.endpoint_descriptors()
                {
                    if endpoint.direction() == rusb::Direction::In
                    {
                        ep_in_num = endpoint.address();
                    }
                    else if endpoint.direction() == rusb::Direction::Out
                    {
                        ep_out_num = endpoint.address();
                    }
                }
                
                break;
            }
        }
    }
    
    if iface_num == 0xff || ep_in_num == 0xff || ep_out_num == 0xff
    {
        println!("No valid interfaces found?");
        return None;
    }
    
    return Some((handle_unwrap, iface_num, ep_in_num, ep_out_num));
}

fn process_input(input_buf: &[u8; 64], n: usize)
{
    if input_buf[0] == 1 {
        let mut reader = Cursor::new(input_buf.clone());
        let pkt: UsbCmdPacket = UsbCmdPacket::read(&mut reader).unwrap();

        if pkt.pkt_len >= 1 {
            if pkt.data[0] == 0xFF
            {

            }
        }
        else
        {
            print!("Received cmd stream... ");
            for i in 0..pkt.pkt_len
            {
                print!("{:02x} ", pkt.data[i as usize]);
            }
            println!("");
        }
    }
    else
    {
        let mut null_term = 0;
        loop
        {
            if null_term >= n {
                break;
            }

            if input_buf[null_term] == 0 {
                break
            }
            null_term += 1;
        }
        
        let read_str = str::from_utf8(&input_buf[..null_term]);
        if read_str.is_ok() {
            let read_str_good = str::replace(read_str.unwrap(), "\r\n", "\n");
            print!("{}", read_str_good);
        }
    }
}

fn run_device(handle: &mut rusb::DeviceHandle<rusb::GlobalContext>, ep_in_num: u8, ep_out_num: u8) -> bool
{
    let mut input_buf: [u8; 64] = [0; 64];
    
    match handle.read_bulk(ep_in_num, &mut input_buf, time::Duration::from_millis(1)) {
        Err(e) => {
            if e == rusb::Error::NoDevice {
                return false;
            }
            //println!("read err {}", e);
        },
        Ok(n) => {
            //println!("Read {} bytes", n);
            
            if n >= 1 {
                process_input(&input_buf, n);
            }
        },
    };

    let ch = getch();
    let try_chr = char::from_u32(ch as u32);
    if !try_chr.is_some() || ch == 0x19a { // don't send resize
        return true;
    }
    let ch_str = &format!("{}", try_chr.unwrap());
    /*for i in 0..ch_str.len() {
        println!("{:x}", ch_str.as_bytes()[i]);
    }*/
    
    match handle.write_bulk(ep_out_num, ch_str.as_bytes(), time::Duration::from_millis(10)) {
        Err(_e) => {
            println!("write err {}", _e);
        },
        Ok(_n) => {
            //println!("Sent {} bytes", n);
        },
    };
    
    return true;
}

fn main() -> Result<(), Error>
{
    let term_now = Arc::new(AtomicBool::new(false));

    for sig in TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    /* Setup ncurses. */
    initscr();

    /* Allow for extended keyboard (like F1). */
    keypad(stdscr(), true);
    noecho();
    nodelay(stdscr(), true);
    scrollok(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    
    println!("Searching for device...");
    while !term_now.load(Ordering::Relaxed)
    {
        let handle_try = find_device();
        if !handle_try.is_some() {
            thread::sleep(time::Duration::from_millis(100));
            refresh();
            continue;
        }
        
        wclear(stdscr());
        
        let handle_dat = handle_try.unwrap();
        let mut handle = handle_dat.0;
        let iface_num = handle_dat.1;
        let ep_in_num = handle_dat.2;
        let ep_out_num = handle_dat.3;
        
        println!("Connected!\n----------");
        
        let try_reset = handle.reset();
        if try_reset.is_err()
        {
            println!("Failed to reset device, exiting...");
            return Ok(());
        }
        
        let try_claim = handle.claim_interface(iface_num);
        if try_claim.is_err()
        {
            println!("Failed to claim interface, exiting...");
            return Ok(());
        }
        
        let magic_data: [u8; 4] = [0x0f, 0xf0, 0x0f, 0xf0];
        match handle.write_bulk(ep_out_num, &magic_data, time::Duration::from_millis(100)) {
            Err(_e) => {
                println!("write err {}", _e);
                continue;
            },
            Ok(_n) => {
                //println!("Sent {} bytes", n);
            },
        };
        
        while !term_now.load(Ordering::Relaxed)
        {
            if !run_device(&mut handle, ep_in_num, ep_out_num)
            {
                println!("Lost connection with device, attempting reconnect...");
                break;
            }
            refresh();
        }
    }
    
    endwin();

    Ok(())
}
