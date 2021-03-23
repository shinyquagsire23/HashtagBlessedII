/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::vm::vsdmmc::*;
use crate::io::uart::*;
use crate::vm::funcs::*;
use crate::arm::threading::*;
use crate::hos::smc::*;
use crate::arm::exceptions::*;
use crate::vm::vmmu::*;
use crate::vm::vsvc::*;
use crate::util::*;
use crate::arm::mmu::*;
use alloc::string::String;
use crate::exception_handler::*;

pub struct VMMIORegRW
{
    addr: u64,
    val: u64,
    debug_print: bool,
    is_write: bool,
    abort_access: bool
}

pub fn vmmio_init()
{
    vsdmmc_init();
}

pub fn vmmio_handle_lowerel_dabt(iss: u32, ctx: &mut [u64]) -> u64
{
    let sas = (iss >> 22) & 0x3;
    let isv = (iss & bit!(24)) != 0;
    let sse = (iss & bit!(21)) != 0;
    let srt = (iss >> 16) & 0x1F;
    let sf = (iss & bit!(15)) != 0;
    let ar = (iss & bit!(14)) != 0;
    let ea = (iss & bit!(9)) != 0;
    let cm = (iss & bit!(8)) != 0;
    let s1ptw = (iss & bit!(7)) != 0;
    let wnr = (iss & bit!(6)) != 0;
    let mut iss_string = String::new();

    let io_addr = get_fipa_el2();
    let is_xzr = (srt == 31);
    let num_bits = 8 << sas;

    let mut wait_dma = false;

    let mut v_regrw = VMMIORegRW {
        addr: io_addr,
        val:0,
        debug_print: false,
        is_write: wnr,
        abort_access: false
    };

    if (is_xzr)
    {
        v_regrw.val = 0;
    }
    else if (v_regrw.is_write && num_bits == 32)
    {
        v_regrw.val = (ctx[srt as usize] & 0xFFFFFFFF);
    }
    else if (v_regrw.is_write && num_bits == 16)
    {
        v_regrw.val = (ctx[srt as usize] & 0xFFFF);
    }
    else if (v_regrw.is_write && num_bits == 8)
    {
        v_regrw.val = (ctx[srt as usize] & 0xFF);
    }

    //SDMMC regs
    if (io_addr >= 0x700b0000 && io_addr <= 0x700c0000)
    {
        v_regrw.debug_print = false;
        //vsdmmc_handle_pre(&v_regrw); // TODO
    }
    else if (io_addr >= UART_PADDR as u64 && io_addr < (UART_PADDR+0x40) as u64)
    {
        if (io_addr != UART_PADDR as u64 && !v_regrw.is_write) {
            v_regrw.abort_access = true;
        }
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x50040000 && io_addr < 0x50042000)
    {
        /*if (io_addr == 0x50041F00 && v_regrw.is_write)
        {
            send_interrupt((u32)ctx[srt]);
            return get_elr_el2() + 4;
        }*/
    }
    else if ((io_addr >= 0x7d000000 && io_addr < 0x7d005800)
             || (io_addr >= 0x70090000 && io_addr < 0x700a0000)
             || (io_addr >= 0x700d0000 && io_addr < 0x700da000))
    {
        //printf("(core %u) attempted to write USB regs! dropping...\n\r", get_core());
        //v_regrw.abort_access = true;
        if (io_addr == 0x7009f004 && v_regrw.is_write) {
            v_regrw.abort_access = true;
        }
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x7000c000 && io_addr < 0x7000d200) // I2C
    {
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x60006000 && io_addr < 0x60007000) // CAR
    {
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x60005000 && io_addr < 0x60005400) // TMR
    {
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x6000d000 && io_addr < 0x6000d800) // GPIO
    {
        v_regrw.debug_print = true;
    }
    else if (io_addr >= 0x70003000 && io_addr < 0x70004000) // PINMUX
    {
        v_regrw.debug_print = true;
    }
    else if (io_addr >= 0x7001b000 && io_addr < 0x7001c000) // EMC
    {
        //if (io_addr == 0x7001b000)
        //    printf("(core %u) mem training?\n\r", get_core());
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x7001e000 && io_addr < 0x70020000) // EMC idk
    {
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x70110000 && io_addr < 0x70110400) // DVFS
    {
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x7000f800 && io_addr < 0x7000fc00) // FUSE
    {
        v_regrw.debug_print = false;
    }
    else if (io_addr >= 0x01000000 && io_addr < 0x30000000) // PCIE
    {
        v_regrw.debug_print = false;
    }

    if (is_xzr)
    {
        v_regrw.val = 0;
    }

    if (!v_regrw.abort_access && num_bits == 32)
    {
        if (v_regrw.is_write) {
            poke32(io_addr, (v_regrw.val & 0xFFFFFFFF) as u32);
        }
        else {
            v_regrw.val = peek32(io_addr) as u64;
        }
    }
    else if (!v_regrw.abort_access && num_bits == 16)
    {
        if (v_regrw.is_write) {
            poke16(io_addr, (v_regrw.val & 0xFFFF) as u16);
        }
        else {
            v_regrw.val = peek16(io_addr) as u64;
        }
    }
    else if (!v_regrw.abort_access && num_bits == 8)
    {
        if (v_regrw.is_write) {
            poke8(io_addr, (v_regrw.val & 0xFF) as u8);
        }
        else {
            v_regrw.val = peek8(io_addr) as u64;
        }
    }

    // Post handlers...

    if (!v_regrw.is_write && !is_xzr)
    {
        ctx[srt as usize] = v_regrw.val;
    }

    //if (v_regrw.debug_print)
    {
        iss_string = get_dabt_iss_str(iss, ctx);

        unsafe
        {
        let mut contextidr: u64 = 0;
        asm!("mrs {0}, CONTEXTIDR_EL1", out(reg) contextidr);
        let pid: u8 = (contextidr & 0xFF) as u8;

        //if (pid >= 0x50)
        println!("core {}: DABT (lower EL, pid {:02x} {}) {}", get_core(), pid, vsvc_get_pid_name(&pid), iss_string);
        }
    }
    
    return get_elr_el2() + 4;
}
