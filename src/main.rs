#![no_std]
#![no_main]
#![feature(global_asm)]
#![allow(unused_parens)]
#![allow(unused)]
#![allow(non_snake_case)]

use io::uart::*;
use io::uart::UARTDevicePort::*;
use core::panic::PanicInfo;

#[macro_use] mod util;

#[macro_use]
extern crate lazy_static;

mod io;
mod hos;

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn not_main() 
{
    let mut uart_a: UARTDevice = UARTDevice::new(UartA, 115200);
    
    uart_a.writeStr("Waddup from EL2\n\r")
}

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}
