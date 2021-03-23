/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use crate::arm::threading::*;
use crate::arm::exceptions::*;
use crate::vm::vsysreg::*;
use crate::vm::vsvc::*;
use crate::vm::vmmio::*;
use crate::vm::vsmc::*;
use crate::arm::mmu::*;
use crate::io::timer::*;
use crate::util::*;
use crate::logger::*;
use alloc::string::String;

pub const EC_WFIWFE:        u8 = (0x01);
pub const EC_ASIMD:         u8 = (0x07);
pub const EC_SVC_A32:       u8 = (0x11);
pub const EC_HVC_A32:       u8 = (0x12);
pub const EC_SMC_A32:       u8 = (0x13);
pub const EC_SVC_A64:       u8 = (0x15);
pub const EC_HVC_A64:       u8 = (0x16);
pub const EC_SMC64:         u8 = (0x17);
pub const EC_MSRMRS:        u8 = (0x18);
pub const EC_IABT_LOWER_EL: u8 = (0x20);
pub const EC_IABT_CUR_EL:   u8 = (0x21);
pub const EC_PC_ALIGN:      u8 = (0x22);
pub const EC_DABT_LOWER_EL: u8 = (0x24);
pub const EC_DABT_CUR_EL:   u8 = (0x25);

pub const fn get_ifsc_dfsc_str<'a>(iss: &'a u32) -> &'a str
{
    let dfsc = *iss & 0x1F;

    return match (dfsc)
    {
        0b000000 => "Address size fault, 0th level",
        0b000001 => "Address size fault, 1st level",
        0b000010 => "Address size fault, 2nd level",
        0b000011 => "Address size fault, 3rd level",
        0b000100 => "Translation fault, 0th level.",
        0b000101 => "Translation fault, 1st level.",
        0b000110 => "Translation fault, 2nd level.",
        0b000111 => "Translation fault, 3rd level.",
        0b001000 => "Access flag fault, 0th level.",
        0b001001 => "Access flag fault, 1st level.",
        0b001010 => "Access flag fault, 2nd level.",
        0b001011 => "Access flag fault, 3rd level.",
        0b001101 => "Permission fault, 1st level.",
        0b001110 => "Permission fault, 2nd level.",
        0b001111 => "Permission fault, 3rd level.",
        0b010000 => "Synchronous external abort.",
        0b011000 => "Synchronous parity error on memory access.",
        0b010100 => "Synchronous external abort on translation table walk, 0th level.",
        0b010101 => "Synchronous external abort on translation table walk, 1st level.",
        0b010110 => "Synchronous external abort on translation table walk, 2nd level.",
        0b010111 => "Synchronous external abort on translation table walk, 3rd level.",
        0b011101 => "Synchronous parity error on memory access on translation table walk, 1st level.",
        0b011110 => "Synchronous parity error on memory access on translation table walk, 2nd level.",
        0b011111 => "Synchronous parity error on memory access on translation table walk, 3rd level.",
        0b100001 => "Alignment fault.",
        0b100010 => "Debug event.",
        
        _ => "Unknown IFSC/DFSC",
    }
}

pub fn get_dabt_iss_str(iss: u32, ctx: &[u64]) -> String
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
    
    let dfsc_str = get_ifsc_dfsc_str(&iss);

    let num_bits = 8 << sas;
    
    let mut value = ctx[srt as usize];
    if (srt == 31) {
        value = 0;
    }
    
    if (num_bits == 16) {
        value &= 0xFFFF;
    }
    else if (num_bits == 8) {
        value &= 0xFF;
    }
    
    let tmp = format!("{}-bit {} {}{:02} ({:08x}), addr {:016x}", num_bits, if wnr { "str" } else { "ldr" }, if sf { 'X' } else { 'W' }, srt, value, get_fipa_el2());
    
    return format!("{}{}{}", dfsc_str, if isv { ", " } else { "" }, if isv { tmp } else { String::new() });
}

pub fn get_iabt_iss_str(iss: u32) -> String
{
    let ea = (iss & bit!(9)) != 0;
    let s1ptw = (iss & bit!(7)) != 0;
    
    let ifsc_str = get_ifsc_dfsc_str(&iss);
    
    return format!("{}, {}", ifsc_str, if s1ptw { "Stage 2 Translation" } else { "Not Stage 2" });
}

pub fn get_mrsmsr_iss_str(iss: u32) -> String
{
    let cv = (iss & bit!(24)) != 0;
    let cond = (iss >> 20) & 0xF;
    let opc2 = (iss >> 17) & 0x7;
    let opc1 = (iss >> 14) & 0x7;
    let crn  = (iss >> 10) & 0xF;
    let rt   = (iss >> 5) & 0x1F;
    let crm  = (iss >> 1) & 0xF;
    let is_read = (iss & bit!(0)) != 0;
    
    if (is_read) {
        return format!("mrs p15, {}, X{}, c{}, c{}, {}", opc1, rt, crn, crm, opc2);
    }
    else {
        return format!("msr p15, {}, X{}, c{}, c{}, {}", opc1, rt, crn, crm, opc2);
    }
}

pub fn print_context(ctx: &[u64], is_dabt: bool)
{
    let esr_el2 = (get_esr_el2() & 0xFFFFFFFF);
    println!("esr_el2: {:08x} {}", esr_el2, vsvc_get_curpid_name());
    println!("elr_el2: {:016x}", get_elr_el2());
    println!("elr_el1: {:016x}", get_elr_el1());

    println!("x0  {:016x} x1  {:016x} x2  {:016x} x3  {:016x} ", ctx[0], ctx[1], ctx[2], ctx[3]);
    println!("x4  {:016x} x5  {:016x} x6  {:016x} x7  {:016x} ", ctx[4], ctx[5], ctx[6], ctx[7]);
    println!("x8  {:016x} x9  {:016x} x10 {:016x} x11 {:016x} ", ctx[8], ctx[9], ctx[10], ctx[11]);
    println!("x12 {:016x} x13 {:016x} x14 {:016x} x15 {:016x} ", ctx[12], ctx[13], ctx[14], ctx[15]);
    println!("x16 {:016x} x17 {:016x} x18 {:016x} x19 {:016x} ", ctx[16], ctx[17], ctx[18], ctx[19]);
    println!("x20 {:016x} x21 {:016x} x22 {:016x} x23 {:016x} ", ctx[20], ctx[21], ctx[22], ctx[23]);
    println!("x24 {:016x} x25 {:016x} x26 {:016x} x27 {:016x} ", ctx[24], ctx[25], ctx[26], ctx[27]);
    println!("x28 {:016x}", ctx[28]);
    println!("sp  {:016x} lr  {:016x} pc  {:016x}", ctx[29], ctx[30], ctx[31]-(if is_dabt { 4 } else { 0 }));
    println!("");
    println!("spsr_el2   {:016x} tpidr {:016x} {:016x}", ctx[32], ctx[33], ctx[34]);
    println!("esr_el1    {:016x} afsr0 {:016x} afsr1 {:016x}", get_esr_el1(), get_afsr0_el1(), get_afsr1_el1());
    
    unsafe
    {
    let mut midr: u64 = 0;
    let mut mpidr: u64 = 0;
    let mut contextidr: u64 = 0;
    asm!("mrs {0},MIDR_EL1", out(reg) midr);
    asm!("mrs {0},MPIDR_EL1", out(reg) mpidr);
    asm!("mrs {0}, CONTEXTIDR_EL1", out(reg) contextidr);
    println!("midr       {:016x} mpidr {:016x} elr_el1 {:016x}", midr, mpidr, get_elr_el1());
    let pid: u8 = (contextidr & 0xFF) as u8;
    println!("contextidr {:016x} ({})", contextidr, vsvc_get_pid_name(&pid));
    }
    
    println!("----");
    println!("");
}

pub fn print_exception(ec: u8, iss: u32, ctx: &[u64], ret_addr_in: u64) -> u64
{
    let mut ec_unk = false;
    let mut iss_unk = true;
    let mut is_dabt = false;
    let mut is_dabt_lower = true;
    //char iss_string[0x200];
    let mut iss_string = String::new();
    let mut ec_string = "unknown exception code";
    
    let mut ret_addr = ret_addr_in;

    //iss_string[0] = 0;
    
    //mutex_lock(&exception_print_mutex);
    

    match (ec)
    {
        EC_WFIWFE => {
            ec_string = "WFI/WFE";
        }

        3 => {
            ec_string = "CP15 MCR/MRC";
        }

        4 => {
            ec_string = "CP15 MCRR/MRRC";
        }

        5 => {
            ec_string = "CP14 MCR/MRC";
        }

        6 => {
            ec_string = "CP14 LDC/STC";
        }

        EC_ASIMD => {
            ec_string = "ASIMD";
        }

        8 => {
            ec_string = "CP10 MRC/VMRS";
        }

        0xC => {
            ec_string = "CP14 MCRR/MRRC";
        }

        0xE => {
            ec_string = "PSTATE.IL";
        }

        EC_SVC_A32 => {
            ec_string = "SVC (AArch32)";
        }

        EC_HVC_A32 => {
            ec_string = "HVC (AArch32)";
        }

        EC_SMC_A32 => {
            ec_string = "SMC (AArch32)";
        }

        EC_SVC_A64 => {
            ec_string = "SVC (AArch64)";
        }

        EC_HVC_A64 => {
            ec_string = "HVC (AArch64)";
        }

        EC_SMC64 => {
            ec_string = "SMC (AArch64)";
        }

        EC_MSRMRS => {
            ec_string = "MSR/MRS (AArch64)";
            iss_string = get_mrsmsr_iss_str(iss);
            iss_unk = false;
        }

        0x19 => {
            ec_string = "SVE";
        }

        0x1f => {
            ec_string = "EL3 IMP DEF";
        }

        EC_IABT_LOWER_EL => {
            ec_string = "IABT (lower EL)";
            iss_string = get_iabt_iss_str(iss);
            iss_unk = false;
            ret_addr = 0;
        }

        EC_IABT_CUR_EL => {
            ec_string = "IABT (current EL)";
            iss_string = get_iabt_iss_str(iss);
            iss_unk = false;
            ret_addr = 0;
        }

        EC_PC_ALIGN => {
            ec_string = "PC Alignment";
            ret_addr = 0;
        }

        EC_DABT_LOWER_EL => {
            ec_string = "DABT (lower EL)";
            iss_string = get_dabt_iss_str(iss, ctx);
            iss_unk = false;
            is_dabt = true;
            is_dabt_lower = true;

            /*if (*(u32*)translate_el1_stage12(ctx[31]) == 0xb8408D2A)
            {
                ctx[9] += 4;
                ctx[10] = *(u32*)0x7001b000;
                ret_addr = get_elr_el2() + 4;
                return;
            }
            else if (*(u32*)translate_el1_stage12(ctx[31]) == 0xb8414D2A)
            {
                ctx[10] = *(u32*)(translate_el1_stage12(ctx[9]));
                ctx[9] += 4;
                ret_addr = get_elr_el2() + 4;
                return;
            }*/

            ret_addr = get_elr_el2() + 4;
            //return;
        }

        EC_DABT_CUR_EL => {
            ec_string = "DABT (current EL)";
            iss_string = get_dabt_iss_str(iss, ctx);
            iss_unk = false;
            is_dabt = true;
            ret_addr = 0;
        }

        0x26 => {
            ec_string = "SP Alignment";
            ret_addr = 0;
        }

        0x28 => {
            ec_string = "FP (AArch32)";
        }

        0x2C => {
            ec_string = "FP (AArch64)";
        }

        0x2F => {
            ec_string = "SError";
            ret_addr = 0;
        }

        0x30 => {
            ec_string = "Breakpoint (lower EL)";
        }

        0x31 => {
            ec_string = "Breakpoint (current EL)";
        }

        0x32 => {
            ec_string = "Software Step (lower EL)";
        }

        0x33 => {
            ec_string = "Software Step (current EL)";
        }

        0x34 => {
            ec_string = "Watchpoint (lower EL)";
        }

        0x35 => {
            ec_string = "Watchpoint (current EL)";
        }

        0x38 => {
            ec_string = "BKPT (AArch32)";
        }

        0x3A => {
            ec_string = "Vector catch (AArch32)";
        }

        0x3C => {
            ec_string = "BRK (AArch64)";
        }
        
        _ => {
            iss_unk = true;
            is_dabt = false;
            ec_unk = true;
            ret_addr = 0;//get_elr_el2() + 4;
            ec_string = "unknown exception code";
        }
    }

    println!("");
    println!("");

    println!("Exception occurred on core {}: {} {}{}{}", get_core(), ec_string, if iss_unk { "" } else { "(" }, iss_string, if iss_unk { "" } else { ")" });
    
    if (ec_unk || iss_unk)
    {
        println!("EC: {:02x}, ISS: {:07x}", ec, iss);
    }
    
    print_context(ctx, is_dabt);  
    
    println!("translate {:016x} -> {:016x} (stage 1 {:016x})", ctx[8], translate_el1_stage12(ctx[8]), translate_el1_stage1(ctx[8]));
    println!("translate {:016x} -> {:016x} (stage 1 {:016x}) {:x}", ctx[10], translate_el1_stage12(ctx[10]), translate_el1_stage1(ctx[10]), peek64(translate_el1_stage12(ctx[10])));
    //println!("translate {:016x} -> {:016x}\n\r", ctx[31], translate_el1_stage12(ctx[31]));

    if (is_dabt_lower)
    {
        let pc_dump = ctx[19]-16;//get_elr_el1() - 16;//ctx[31]-16;
        println!("translated PC {:016x}", translate_el1_stage12(pc_dump));
        println!("");
        if (translate_el1_stage12(pc_dump) >= 0x80000000 && translate_el1_stage12(pc_dump) < 0x200000000) {
            hexdump("dump @ PC ", translate_el1_stage12(pc_dump), 0x60);
        }
    }
    
    return ret_addr;
}

pub fn handle_exception(which: i32, ctx: &mut [u64]) -> u64
{
    let esr_el2: u32 = (get_esr_el2() & 0xFFFFFFFF);
    let esr_el1: u32 = (get_esr_el1() & 0xFFFFFFFF);
    let mut ec: u8 = (esr_el2 >> 26) as u8;
    let mut iss: u32 = esr_el2 & 0x1FFFFFF;

    let mut ret_addr: u64 = get_elr_el2() + 4;
    /*if (get_core() == 3)
    {
        last_core_ret = ret_addr;
        last_core_name = vsvc_get_curpid_name();
    }*/

    let start_ticks: u64 = vsysreg_getticks();
    let mut end_ticks: u64 = start_ticks;
    
    if (ec == EC_HVC_A64) // HVC
    {
        let hvc_num: u8 = (iss & 0xFF) as u8;
        ec = (esr_el1 >> 26) as u8;
        iss = esr_el1 & 0x1FFFFFF;

        if (hvc_num == 1)
        {
            // emulate ff 42 03 d5     msr        DAIFClr,#0x2
            unsafe
            {
            let mut spsr_el2: u64 = 0;
            asm!("mrs {0}, spsr_el2", out(reg) spsr_el2);
            spsr_el2 &= !0x80;
            asm!("msr spsr_el2, {0}", in(reg) spsr_el2);
            }

            //TODO
            ret_addr = vsvc_pre_handle(iss, ctx);
        }
        else if (hvc_num == 2) // SVC post-hook
        {
            // emulate df 42 03 d5     msr        DAIFSet,#0x2
            unsafe
            {
            let mut spsr_el2: u64 = 0;
            asm!("mrs {0}, spsr_el2", out(reg) spsr_el2);
            spsr_el2 |= 0x80;
            asm!("msr spsr_el2, {0}", in(reg) spsr_el2);
            }

            //TODO
            ret_addr = vsvc_post_handle(iss, ctx);
        }
        else if (hvc_num == 3)
        {
            // emulate ff 42 03 d5     msr        DAIFClr,#0x2
            unsafe
            {
            let mut spsr_el2: u64 = 0;
            asm!("mrs {0}, spsr_el2", out(reg) spsr_el2);
            spsr_el2 &= !0x80;
            asm!("msr spsr_el2, {0}", in(reg) spsr_el2);
            }

            println!("(core {}) Unsupported SVC A32 hook!!", get_core());
            ret_addr = get_elr_el2();
        }
        else if (hvc_num == 4)
        {
            // emulate df 42 03 d5     msr        DAIFSet,#0x2
            unsafe
            {
            let mut spsr_el2: u64 = 0;
            asm!("mrs {0}, spsr_el2", out(reg) spsr_el2);
            spsr_el2 |= 0x80;
            asm!("msr spsr_el2, {0}", in(reg) spsr_el2);
            }

            println!("(core {}) Unsupported SVC A32 hook!!", get_core());
            ret_addr = get_elr_el2();
        }
        else if (ec == EC_DABT_LOWER_EL || ec == EC_IABT_LOWER_EL || ec == EC_PC_ALIGN)
        {
/*
            println!("");
            println!("");
            printf("Lower EL intercept, stack dump:\n\r");
            u64 stack = translate_el1_stage12(ctx[29]);
            printf("sp translated{:016x}\n\r", stack);
            for i in 0..0x100
            {
                printf("{:016x}: {:016x}\n\r", ctx[29]+i*8, *(u64*)(stack+i*8));
            }
            
            
            u64 lr = translate_el1_stage12(ctx[20]);
            printf("lr translated {:016x}\n\r", lr);
            for i in 0..0x40
            {
                printf("{:016x}: {:016x}\n\r", ctx[30]+i*sizeof(u64), *(u64*)(lr+i*sizeof(u64)));
            }
*/
            if (((iss & bit!(24)) != 0))
            {
                ret_addr = vmmio_handle_lowerel_dabt(iss, ctx);
            }
            else
            {
                ret_addr = get_elr_el2();
                ret_addr = print_exception(ec, iss, ctx, ret_addr);
            }
        }
        else if (ec == EC_SVC_A32 || ec == EC_SVC_A64)
        {
            //return vsvc_pre_handle((u8)iss, ctx);
            ret_addr = get_elr_el2();
        }
        else if (ec == EC_ASIMD)
        {
            //printf("(core {}) ASIMD sync IRQ, a process probably started\n\r", get_core());
            ret_addr = get_elr_el2();
        }
        else
        {
            println!("(core {}) ec {:x} {:016x}", get_core(), ec, get_elr_el2());

            ret_addr = get_elr_el2();
        }
        
        
        if (hvc_num == 0 && ec != 0x15)
        {
            println!("(core {}) ec {:x} {:016x}", get_core(), ec, get_elr_el2());
            ctx[17] = ctx[16] & 0x3F;
            ret_addr = get_elr_el2();
        }
        //mutex_unlock(&exception_print_mutex);
    }
    else if (ec == EC_MSRMRS)
    {
        //print_exception(ec, iss, ctx, ret_addr);
        ret_addr = vsysreg_handle(iss, ctx); // TODO
    }
    else if (ec == EC_SMC64)
    {
        if (ctx[0] == 0x00000000c3000002) {
        //print_exception(ec, iss, ctx, ret_addr);
        }
        ret_addr = vsmc_handle(iss, ctx);
    }
    else if (ec == EC_DABT_LOWER_EL && ((iss & bit!(24)) != 0))
    {
        ret_addr = vmmio_handle_lowerel_dabt(iss, ctx);
    }
    else
    {
        ret_addr = print_exception(ec, iss, ctx, ret_addr);
    }

    if (ret_addr == 0)
    {
        logger_unsafe_override();
        log_process();
        println_unsafe!("Resetting...");
        //uart_shutdown(UART_A);
        
        //if (!get_core()) {
        //    loop{}
        //}

        for i in 0..10
        {
            println_unsafe!("Terminating in {} seconds...", 10-i);
/*
            printf("translate {:016x} -> {:016x} (stage 1 {:016x})\n\r", ctx[9], translate_el1_stage12(ctx[9]), translate_el1_stage1(ctx[9]));
            printf("translate {:016x} -> {:016x} (stage 1 {:016x})\n\r", ctx[8], translate_el1_stage12(ctx[8]), translate_el1_stage1(ctx[8]));
            hexdump("dump @ PC ", translate_el1_stage12(ctx[31]), 0x60);
*/
            timer_wait(1000000);
        }
        unsafe { t210_reset(); }
        loop{}
    }

    end_ticks = vsysreg_getticks();
    vsysreg_addoffset(end_ticks - start_ticks);
    //mutex_unlock(&exception_print_mutex);
    return ret_addr;
}
