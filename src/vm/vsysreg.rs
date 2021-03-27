/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::arm::exceptions::*;
use crate::arm::threading::*;
use crate::arm::mmu::*;
use crate::exception_handler::*;
use crate::util::*;

static mut HAS_HOOKED_EXCEPTIONS: bool = false;

pub fn vsysreg_getticks() -> u64
{
    sysreg_read!("cntpct_el0")
}

pub fn vsysreg_getticks_scaled(ticks: u64) -> u64
{
    return ticks; // TODO?
}

pub fn vsysreg_getticks_unscaled(ticks: u64) -> u64
{
    return ticks; // TODO?
}

pub fn vsysreg_addoffset(offs: u64)
{
    // TODO
}

pub fn vsysreg_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    unsafe
    {
    let cv = (iss & bit!(24)) != 0;
    let cond = (iss >> 20) & 0xF;
    let opc2 = (iss >> 17) & 0x7;
    let opc1 = (iss >> 14) & 0x7;
    let crn  = (iss >> 10) & 0xF;
    let rt   = (iss >> 5) & 0x1F;
    let crm  = (iss >> 1) & 0xF;
    let is_read = (iss & bit!(0)) != 0;

    let iss_string = get_mrsmsr_iss_str(iss);

    let retaddr = get_elr_el2() + 4;

    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.100403_0200_00_en/obp1469702473493.html
    if (is_read)
    {
        let mut val: u64 = 0;
        if (opc1 == 1 && crn == 0 && crm == 0 && opc2 == 1)
        {
            val = sysreg_read!("clidr_el1");
            println!("(core {}) CLIDR_EL1 {:016x}", get_core(), val);
        }
        else if (opc1 == 3 && crn == 14 && crm == 0 && opc2 == 1)
        {
            val = vsysreg_getticks_scaled(vsysreg_getticks());
            //println!("(core {}) CNTPCT_EL0 %016llx {:016x}", get_core(), val, modified_count);
        }
        else if (opc1 == 3 && crn == 14 && crm == 2 && opc2 == 1)
        {
            val = sysreg_read!("cntp_ctl_el0");
            //println!("(core {}) CNTP_CTL_EL0 {:016x}", get_core(), val);
        }
        else if (opc1 == 3 && crn == 14 && crm == 2 && opc2 == 2)
        {
            val = vsysreg_getticks_scaled(sysreg_read!("cntp_cval_el0"));
            //println!("(core {}) CNTP_CVAL_EL0 {:016x}", get_core(), val);
        }
        else {
            println!("{} {:016x}", iss_string, retaddr);
        }
        ctx[rt as usize] = val;
    }
    else
    {
        let val = ctx[rt as usize];
        if (opc1 == 3 && crn == 7 && crm == 10 && opc2 == 1)
        {
            asm!("dc cvac, {0}", in(reg) val);
        }
        else if (opc1 == 0 && crn == 2 && crm == 0 && opc2 == 0)
        {
            sysreg_write!("ttbr0_el1", val);
            println!("(core {}) TTBR0_EL1 {:016x}", get_core(), val);
        }
        else if (opc1 == 0 && crn == 2 && crm == 0 && opc2 == 1)
        {
            sysreg_write!("ttbr1_el1", val);
            println!("(core {}) TTBR1_EL1 {:016x}", get_core(), val);
        }
        else if (opc1 == 0 && crn == 2 && crm == 0 && opc2 == 2)
        {
            sysreg_write!("tcr_el1", val);
            println!("(core {}) TCR_EL1 {:016x}", get_core(), val);
        }
        else if (opc1 == 0 && crn == 10 && crm == 2 && opc2 == 0)
        {
            sysreg_write!("mair_el1", val);
            println!("(core {}) MAIR_EL1 {:016x}", get_core(), val);
        }
        else if (opc1 == 0 && crn == 1 && crm == 0 && opc2 == 0)
        {
            sysreg_write!("sctlr_el1", val);
            println!("(core {}) SCTLR_EL1 {:016x}", get_core(), val);
        }
        else if (opc1 == 0 && crn == 13 && crm == 0 && opc2 == 1)
        {
            sysreg_write!("contextidr_el1", val);
            
            //TODO check address space?
            
            //let mut val_vbar: u64 = 0;
	        //asm!("mrs {0}, VBAR_EL1", out(reg) val_vbar);
	        //printf("(core %u) VBAR_EL1 %016llx\n\r", get_core(), val_vbar);

            /*if (!HAS_HOOKED_EXCEPTIONS && val_vbar != 0)
            {
                let vbar_ptr = translate_el1_stage12(val_vbar);

                // clrex
                // hvc #0
                // b exception_handler
                poke32(vbar_ptr + 0x408, peek32(vbar_ptr + 0x404) - 1); // adjust branch
                poke32(vbar_ptr + 0x404, peek32(vbar_ptr + 0x400)); // clrex first
                poke32(vbar_ptr + 0x400, 0xd4000002 | (0 << 5)); // HVC #0 instruction

                HAS_HOOKED_EXCEPTIONS = true;
            }*/
        }
        else if (opc1 == 3 && crn == 14 && crm == 2 && opc2 == 1)
        {
            sysreg_write!("cntp_ctl_el0", val);
            //println!("(core {}) CNTP_CTL_EL0 {:016x}", get_core(), val);
        }
        else if (opc1 == 3 && crn == 14 && crm == 2 && opc2 == 2)
        {
            sysreg_write!("cntp_cval_el0", vsysreg_getticks_unscaled(val));
            //println!("(core {}) CNTP_CVAL_EL0 {:016x}", get_core(), val);
        }
        else {
            println!("{} {:016x}", iss_string, retaddr);
        }
    }

    //print_context();

    return retaddr;
    }
}
