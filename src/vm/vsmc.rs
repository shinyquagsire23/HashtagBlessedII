/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::vm::funcs::*;
use crate::arm::threading::*;
use crate::hos::smc::*;
use crate::hos::kernel::*;
use crate::arm::exceptions::*;
use crate::vm::vmmu::*;
use crate::util::*;
use crate::arm::mmu::*;
use crate::exception_handler::*;
use crate::io::smmu::*;
use crate::task::*;
use crate::usbd::usbd::irq_usb;

extern "C"
{
    static __text_start: u32;
}

static mut WARM_ENTRYPOINT: u64 = KERNEL_START;
static mut WARM_ARG: u64 = 0;
static mut LAST_RAND: u32 = 0;

fn rand_gen() -> u16
{
    unsafe
    {
        let next = (1103515245 * LAST_RAND) + 12345;
        LAST_RAND = next;
        return ((next >> 16) & 0xffff) as u16;
    }
}

fn rand_gen_32() -> u32
{
    return (rand_gen() as u32) << 16 | (rand_gen() as u32);
}

fn rand_gen_64() -> u64
{
    return (rand_gen_32() as u64) << 32 | (rand_gen_32() as u64);
}

pub fn vsmc_get_warm_entrypoint() -> u64
{
    unsafe
    {
        return WARM_ENTRYPOINT;
    }
}

pub fn vsmc_set_warm_entrypoint(entry: u64, arg: u64)
{
    unsafe
    {
        WARM_ENTRYPOINT = entry;
        WARM_ARG = arg;
    }
}

pub fn vsmc_handle(iss: u32, ctx: &mut [u64]) -> u64
{
    let smc_which = iss & 0x1;
    let retaddr = get_elr_el2() + 4;

    let smc_cmd = ctx[0] | ((smc_which as u64) << 32);
    let smc_arg0 = ctx[1];
    let smc_arg1 = ctx[2];



    /*u8 argType = (smc_cmd >> 8) & 0xFF;
    for (int i = 0; i < 8; i++)
    {
        if (argType & BIT(i))
            ctx[i] = ipaddr_to_paddr(ctx[i]);
    }*/
    let mut silence_print = false;

    if (smc_cmd == SMC_CPUON) // CPUOn
    {
        unsafe
        {
        vsmc_set_warm_entrypoint(ctx[2], ctx[3]);
        ctx[2] = (to_u64ptr!(&__text_start)) + 4;
        }
        //if (ctx[1] == 3)
        //disable_smcstuff();
    }
    else if (smc_cmd == SMC0_LOADAESKEY)
    {
        silence_print = true;
    }


    if (smc_cmd == SMC_RWREGISTER && smc_arg0 >= MC_BASE && smc_arg0 < MC_END)
    {
        if smmu_handle_rwreg(ctx) {
            return retaddr;
        }
        silence_print = true;
    }
    else if (smc_cmd == SMC_CONFIGURECARVEOUT)
    {
        ctx[2] = ipaddr_to_paddr(ctx[2]);
    }
    else if (smc_cmd == SMC_RWREGISTER)
    {
        silence_print = true;
    }
    else if (smc_cmd == SMC0_GETRESULT || smc_cmd == SMC0_GETRESULTDATA)
    {
        silence_print = true;
    }
    else if (smc_cmd == SMC0_GENAESKEK || smc_cmd == SMC0_COMPUTEAES || smc_cmd == SMC0_COMPUTECMAC || smc_cmd == SMC0_GETCONFIG/* || smc_cmd == SMC_GENRANDOMBYTES || smc_cmd == SMC_GETCONFIG*/)
    {
        silence_print = true;
    }
    else if (smc_cmd == SMC_CPUOFF)
    {
        println_core!("SmcCpuOff called!");
    }
    else if (smc_cmd == SMC_CPUSUSPEND)
    {
        println_core!("SmcCpuSuspend called!");
        crate::io::timer::timer_wait(1000000);
        println_core!("SMC #{} Smc{} (X0 = {:016x}, X1 = {:016x}, X2 = {:016x}, X3 = {:016x})", smc_which, get_smc_name(smc_cmd), ctx[0], ctx[1], ctx[2], ctx[3]);
        
        vsmc_set_warm_entrypoint(ctx[2], ctx[3]);
        unsafe
        {
            ctx[2] = (to_u64ptr!(&__text_start)) + 4;
        }
        
        /*if get_core() == 0 {
            loop 
            {
                irq_usb();
                task_advance();
            }
        } else {
            loop{}
        }*/
    }
    
    if (smc_cmd == SMC_GENRANDOMBYTES)
    {
        let mut i = smc_arg0;
        let mut j = 1;
        
        if i > 0x38 {
            i = 0x38;
        }
        while i > 0
        {
            ctx[j] = rand_gen_64();
            j += 1;
            i -= 8;
        }
        ctx[0] = 0;
        return retaddr;
    }

    if (!silence_print)
    {
        //println!("(core {}) SMC #{} Smc{} (X0 = {:016x}, X1 = {:016x}, X2 = {:016x}, X3 = {:016x})", get_core(), smc_which, get_smc_name(smc_cmd), ctx[0], ctx[1], ctx[2], ctx[3]);
        //println!("          (X4 = {:016x}, X5 = {:016x}, X6 = {:016x}, X7 = {:016x})", ctx[4], ctx[5], ctx[6], ctx[7]);
    }

/*    if (smc_cmd == SMC0_GETCONFIG && ctx[1] == 65000)
    {
        ctx[0] = 0;
        ctx[1] = (0x08000100 | (8<<32) | (0<<40) | (9<<48)|(0<<56));
        return retaddr;
    }
    else if (smc_cmd == SMC0_GETCONFIG && (ctx[1] >= 65001 && ctx[1] <= 65100))
    {
        ctx[0] = 0;
        ctx[1] = 0;
        return retaddr;
    }
    else if (smc_cmd == SMC0_SETCONFIG && (ctx[1] >= 65000 && ctx[1] <= 65100))
    {
        ctx[0] = 0;
        ctx[1] = 0;
        return retaddr;
    }
        // begin whatever the fuck this atmosphere junk is
    else if (smc_cmd == 0xF0000404) // emunand cfg
    {
        memset32(to_u64ptr!(&ctx[0]), 0, 8*8);
        ctx[0] = 0;
        ctx[1] = 0x30534645;
        return retaddr;
    }
    else if (smc_cmd == 0xF0000003) // dram write
    {
        let addr = translate_el1_stage12(ctx[1]);
        let size = ctx[3];
        if (size > 8) {
            ctx[0] = 2;
        }
        else {
            memcpy32(addr, to_u64ptr!(&ctx[2]), size as usize);
        }
        return retaddr;
    }
    else if (smc_cmd == 0xF0000201) // iram cpy
    {
        ctx[0] = 0;
        return retaddr;
    }
    else if (smc_cmd == 0xF0000002) // write reg
    {
        ctx[0] = 0;
        return retaddr;
    }*/
    // end atmosphere junk

    if (smc_cmd == SMC_PANIC) {
        panic!("SMC Panic!");
    }

    if (smc_which == 1) {
        unsafe { smc1_shim(ctx.as_mut_ptr()); }
    }
    else if (smc_which == 0) {
        unsafe { smc0_shim(ctx.as_mut_ptr()); }
    }
    
    if (!silence_print)
    {
        //println!("(core {}) ret SMC #{} Smc{} (X0 = {:016x}, X1 = {:016x}, X2 = {:016x}, X3 = {:016x})", get_core(), smc_which, get_smc_name(smc_cmd), ctx[0], ctx[1], ctx[2], ctx[3]);
        //println!("          (X4 = {:016x}, X5 = {:016x}, X6 = {:016x}, X7 = {:016x})", ctx[4], ctx[5], ctx[6], ctx[7]);
    }

    if (ctx[0] != 0)
    {
        //println!("(core {}) SMC #{} Smc{} returned {:08x}", get_core(), smc_which, get_smc_name(smc_cmd), ctx[0]);
    }
    
    if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_PROGRAMVERIFY)
    {
        ctx[0] = 0;
        ctx[1] |= 1;//1;
    }
    else if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_KERNELCONFIG)
    {
        ctx[0] = 0;
        ctx[1] |= (1 << 8);
    }
    else if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_ISDEBUGMODE)
    {
        ctx[0] = 0;
        ctx[1] = 1;//1;
    }
    else if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_ISRECOVERYBOOT)
    {
        ctx[0] = 0;
        //ctx[1] = 1;
        ctx[1] = 0;
    }
    else if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_HWTYPE)
    {
        ctx[0] = 0;
        ctx[1] = 0;
    }
    else if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_ISRETAIL)
    {
        ctx[0] = 0;
        ctx[1] = 1;
    }
    else if (smc_cmd == SMC_GETCONFIG && smc_arg0 == CONFIGITEM_BOOTREASON)
    {
        ctx[0] = 0;
        ctx[1] = 2;
    }

    return retaddr;
}
