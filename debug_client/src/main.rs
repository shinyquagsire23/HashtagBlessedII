/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![feature(assoc_char_funcs)]

extern crate rusb;

mod file_cmd;
mod app;
mod ui;
mod util;

use std::{
    error::Error,
    io::{stdout, Write},
    sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}},
    thread,
    time::{Duration, Instant},
    time,
    str
};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use binread::{BinRead, io::Cursor};
use crate::file_cmd::file_cmd_handle;
use crate::app::App;
use std::string::String;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, MouseEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

const VID_NINTENDO: u16 = 0x057e;
const PID_SWITCH: u16 = 0x2000;

static mut LOG_BUF: String = String::new();
static mut CMD_BUF: String = String::new();
static mut SPARKLINE_MAX: u64 = 1;
static mut SPARKLINE: u64 = 0;
static mut SPARKLINE_IDX: u64 = 0;

#[derive(BinRead)]
#[br(magic = b"\x01")]
pub struct UsbCmdPacket {
    pkt_len: u8,

    #[br(little, count = pkt_len)]
    data: Vec<u8>,
}

pub struct UsbCtx {
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
    ep_in_num: u8,
    ep_out_num: u8,
    log_buf: String
}

#[macro_use]
macro_rules! println {
    () => { };
    ($fmt:expr) => { 
        unsafe {
        LOG_BUF.push_str($fmt);
        LOG_BUF.push_str("\n");
        }
    };
    ($fmt:expr, $($arg:tt)*) => {{
        let text = format!($fmt, $($arg)*);
        unsafe {
        LOG_BUF.push_str(&text);
        LOG_BUF.push_str("\n");
        }
    }};
}

macro_rules! print {
    () => { };
    ($fmt:expr) => { 
        unsafe {
        LOG_BUF.push_str($fmt);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {{
        let text = format!($fmt, $($arg)*);
        unsafe {
        LOG_BUF.push_str(&text);
        }
    }};
}

enum Event<I> {
    Input(I),
    MouseInput(MouseEvent),
    Tick,
}

pub fn get_sparkline() -> u64
{
    unsafe { SPARKLINE }
}

pub fn get_sparkline_idx() -> u64
{
    unsafe { SPARKLINE_IDX }
}

pub fn get_sparkline_max() -> u64
{
    unsafe { SPARKLINE_MAX }
}

pub fn get_log_buf() -> &'static String
{
    unsafe { &LOG_BUF }
}

pub fn clear_log_buf()
{
    send_cmd(&String::new());
    unsafe { LOG_BUF = String::from("") }
}

pub fn get_cmd() -> Option<String>
{
    unsafe
    {
        if CMD_BUF.is_empty() {
            return None;
        }
        
        return Some(CMD_BUF.clone());
    }
}

pub fn send_cmd(cmd: &String) {
    unsafe
    {
        CMD_BUF = cmd.clone();
    }
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

fn process_input(ctx: &mut UsbCtx, input_buf: &[u8; 64], n: usize)
{
    if input_buf[0] == 1 {
        let mut reader = Cursor::new(input_buf.clone());
        let pkt: UsbCmdPacket = UsbCmdPacket::read(&mut reader).unwrap();

        if pkt.pkt_len >= 1 {
            if pkt.data[0] == 0xFF
            {

            }
            else if pkt.data[0] == 0xFE // sparkline info
            {
                let u32vals = [pkt.data[1], pkt.data[2], pkt.data[3], pkt.data[4]];
                let next_sparkline = u32::from_le_bytes(u32vals) as u64;
                unsafe {
                    if next_sparkline > SPARKLINE_MAX {
                        SPARKLINE_MAX *= 2;
                    }
                    
                    if next_sparkline < SPARKLINE_MAX && SPARKLINE_MAX > 16384 {
                        SPARKLINE_MAX /= 2;
                    }
                    
                    SPARKLINE = next_sparkline;
                    SPARKLINE_IDX += 1;
                }
            }
            else if pkt.data[0] == 0 {
                println!("[Host] Connection is recovered");
            }
            else if pkt.data[0] == 1 {
                file_cmd_handle(ctx, &pkt);
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
        let mut last_newline = 0;
        loop
        {
            if null_term >= n {
                break;
            }

            if input_buf[null_term] == 0 {
                break
            }
            
            if input_buf[null_term] == '\n' as u8 {
                last_newline = null_term;
            }
            null_term += 1;
        }
        
        // the \r\n escape code can sometimes cause lines to get dropped if
        // a packet splits exactly between the two and the replace below doesn't happen,
        // prevent a line from being logged immediately if it ends in \r 
        if null_term != 0 && input_buf[null_term-1] == '\r' as u8 {
            let read_str = str::from_utf8(&input_buf[(last_newline+1)..null_term]);
            if read_str.is_ok() {
                let unwrapped = read_str.unwrap();
                ctx.log_buf = ctx.log_buf.clone() + &String::from(unwrapped);
            }
            null_term = last_newline;
        }
        
        if null_term == 0 {
            return;
        }

        let read_str = str::from_utf8(&input_buf[..null_term]);
        if read_str.is_ok() {
            let unwrapped = ctx.log_buf.clone() + &String::from(read_str.unwrap());
            
            let read_str_good = unwrapped.replace("\r\n", "\n");
            print!("{}", read_str_good);
            ctx.log_buf = String::from("");
        }
    }
}

fn run_device(ctx: &mut UsbCtx) -> bool
{
    let mut input_buf: [u8; 64] = [0; 64];
    
    match ctx.handle.read_bulk(ctx.ep_in_num, &mut input_buf, time::Duration::from_millis(1)) {
        Err(e) => {
            if e == rusb::Error::NoDevice {
                return false;
            }
            //println!("read err {}", e);
        },
        Ok(n) => {
            //println!("Read {} bytes", n);
            
            if n >= 1 {
                process_input(ctx, &input_buf, n);
            }
        },
    };
    
    if let Some(ch_str) = get_cmd() {
        match ctx.handle.write_bulk(ctx.ep_out_num, ch_str.as_bytes(), time::Duration::from_millis(10)) {
            Err(_e) => {
                println!("write err {}", _e);
            },
            Ok(_n) => {
                //println!("Sent {} bytes", n);
            },
        };
        
        send_cmd(&String::new());
    }
    
    return true;
}

fn main() -> Result<(), Box<dyn Error>>
{
    let term_now = Arc::new(AtomicBool::new(false));

    for sig in TERM_SIGNALS {
        flag::register(*sig, Arc::clone(&term_now))?;
    }
    
    
    let enhanced_graphics = true;
    let tick_rate = 10;

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(tick_rate);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                let read_unwrap = event::read().unwrap();
                if let CEvent::Key(key) = read_unwrap {
                    tx.send(Event::Input(key)).unwrap();
                }
                else if let CEvent::Mouse(mouse) = read_unwrap {
                    tx.send(Event::MouseInput(mouse)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new("Crossterm Demo", enhanced_graphics);

    terminal.clear()?;

    let mut draw_term = || -> Result<(), Box<dyn Error>> {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        match rx.recv()? {
            Event::Input(event) => {
                match event.code {
                    KeyCode::Enter => app.on_key('\n'),
                    KeyCode::Char(c) => {
                        if c == 'c' && event.modifiers == KeyModifiers::CONTROL {
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
                            term_now.store(true, Ordering::Relaxed);
                            return Ok(());
                        }
                        app.on_key(c)
                    },
                    KeyCode::Backspace => app.on_backspace(),
                    KeyCode::Delete => app.on_delete(),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Up => app.on_up(),
                    KeyCode::Right => app.on_right(),
                    KeyCode::Down => app.on_down(),
                    KeyCode::PageUp => app.on_pageup(),
                    KeyCode::PageDown => app.on_pagedown(),
                    _ => {}
                }
            },
            Event::MouseInput(event) => {
                match event {
                    MouseEvent::ScrollUp(..) => app.on_pageup(),
                    MouseEvent::ScrollDown(..) => app.on_pagedown(),
                    _ => {}
                }
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            term_now.store(true, Ordering::Relaxed);
            return Ok(());
        }
        
        return Ok(());
    };
    
    println!("Searching for device...");
    draw_term()?;
    while !term_now.load(Ordering::Relaxed)
    {
        draw_term()?;

        let handle_try = find_device();
        if !handle_try.is_some() {
            //thread::sleep(time::Duration::from_millis(100));
            continue;
        }
        
        clear_log_buf();
        draw_term()?;
        
        let handle_dat = handle_try.unwrap();
        let mut handle = handle_dat.0;
        let iface_num = handle_dat.1;
        let ep_in_num = handle_dat.2;
        let ep_out_num = handle_dat.3;
        
        println!("Connected!\n----------");
        draw_term()?;
        
        let try_reset = handle.reset();
        if try_reset.is_err()
        {
            println!("Failed to reset device, exiting...");
            draw_term()?;
            return Ok(());
        }
        
        let try_claim = handle.claim_interface(iface_num);
        if try_claim.is_err()
        {
            println!("Failed to claim interface, exiting...");
            draw_term()?;
            return Ok(());
        }
        
        let magic_data: [u8; 4] = [0x0f, 0xf0, 0x0f, 0xf0];
        match handle.write_bulk(ep_out_num, &magic_data, time::Duration::from_millis(100)) {
            Err(_e) => {
                println!("write err {}", _e);
                draw_term()?;
                continue;
            },
            Ok(_n) => {
                //println!("Sent {} bytes", n);
            },
        };
        
        let mut ctx: UsbCtx = UsbCtx {
            handle: handle,
            ep_in_num: ep_in_num,
            ep_out_num: ep_out_num,
            log_buf: String::new()
        };
        
        if term_now.load(Ordering::Relaxed) {
            break;
        }
        
        while !term_now.load(Ordering::Relaxed)
        {
            draw_term()?;
            if !run_device(&mut ctx)
            {
                println!("Lost connection with device, attempting reconnect...");
                break;
            }
        }
    }

    Ok(())
}
