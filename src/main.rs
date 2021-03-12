#![no_std]
#![no_main]
#![feature(global_asm)]

use io::uart::*;
use core::panic::PanicInfo;

mod io;
mod hos;
mod util;

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn not_main() 
{
    uart_configure_a();
}

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}
