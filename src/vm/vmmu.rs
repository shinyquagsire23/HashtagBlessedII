/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use crate::arm::cache::*;
use crate::vm::funcs::*;
use crate::util::*;
use crate::arm::threading::get_core;

const LV1_RANGE_SIZE: u64 = (0x040000000);
const LV2_RANGE_SIZE: u64 = (0x000200000);
const LV3_RANGE_SIZE: u64 = (0x000001000);

const VTTBR_BLOCK_OR_VAL: u64 = (0x000000000004C5);
const VTTBR_PAGE_OR_VAL_IO: u64 = (0x000000000004C7);
const VTTBR_PAGE_OR_VAL_MEM: u64 = (0x000000000004FF);
// SH[1:0] << 8
// S2AP[1:0] << 6
// MemAttr[3:0] << 2
// we want: write-back cacheable, non-sharable

// normal, gathering, reordering, early write ack

extern "C" {
    static __vttbr_lv1: u32;
    static __vttbr_lv2_slab: u32;
    static __vttbr_lv3_slab: u32;
}

static mut vttbr_lv1: u64 = 0;
static mut vttbr_lv2_slab: u64 = 0;
static mut vttbr_lv3_slab: u64 = 0;
static mut vttbr_lv2_slab_idx: usize = 0;
static mut vttbr_lv3_slab_idx: usize = 0;

pub fn ipaddr_to_paddr(ipaddr: u64) -> u64
{
    let ipaddr_trunc = ipaddr & 0xFFFFFFFFF;
    if (ipaddr_trunc < 0xD0000000) {
        return ipaddr;
    }

    //if (ipaddr_trunc >= 0x84000000 && ipaddr_trunc < 0x86000000)
        //return ipaddr;
    //    return ipaddr = (ipaddr - 0x84000000 + 0xF0000000 );

/*
    if (ipaddr_trunc >= 0x90000000 && ipaddr_trunc < 0x90400000)
        return ipaddr;
    
    if (ipaddr_trunc >= 0xC0000000 && ipaddr_trunc < 0xC0400000)
        return ipaddr;
*/

    let mut paddr = (ipaddr + 0x8000000);
    
/*
    if (paddr >= 0x90000000 && paddr < 0x90400000)
        paddr += 0x400000;
    
    if (paddr >= 0xC0000000 && paddr < 0xC0400000)
        paddr += 0x400000;
*/

    //if (paddr >= 0x84000000 && paddr < 0x86000000)
    //    paddr += 0x2000000;
    
    //if (paddr >= 0xF0000000 && paddr < 0xF2000000)
    //    paddr = (paddr - 0xF0000000 + 0x84000000);

    if (paddr > 0x200000000) {
        paddr = 0;
    }

    return paddr;
}

pub fn vttbr_init()
{
    unsafe
    {
        vttbr_lv1 = to_u64ptr!(&__vttbr_lv1);
        vttbr_lv2_slab = to_u64ptr!(&__vttbr_lv2_slab);
        vttbr_lv3_slab = to_u64ptr!(&__vttbr_lv3_slab);
        
        println!("{:016x} {:016x} {:016x}", vttbr_lv1, vttbr_lv2_slab, vttbr_lv3_slab);
    }
}

pub fn vttbr_new_lv3_pagetable(start_addr: u64) -> u64
{
    unsafe
    {
        let page_ent: u64 = vttbr_lv3_slab + (vttbr_lv3_slab_idx * 0x1000) as u64;
        vttbr_lv3_slab_idx += 1;
        memset32(page_ent, 0, 0x1000);
        
        for i in 0..(0x1000/8)
        {
            let mut target_addr: u64 = start_addr + (i * LV3_RANGE_SIZE) as u64;
            target_addr = ipaddr_to_paddr(target_addr);

            let mut test = 0;
            let or_val = (if target_addr >= 0x80000000 { VTTBR_PAGE_OR_VAL_MEM } else { VTTBR_PAGE_OR_VAL_IO });
            let page_ent_val = or_val | target_addr;
            
            let arr_offs: u64 = (i*8) as u64;
            
            if (target_addr >= 0x80000000 && target_addr < 0x200000000) {
                poke64(page_ent+arr_offs, page_ent_val);
            }
            else if (target_addr >= 0x50044000 && target_addr < 0x50046000)
            {
                poke64(page_ent+arr_offs, 0);
                test = 1;
            }
            else if (target_addr >= 0x50040000 && target_addr < 0x50042000)
            {
                poke64(page_ent+arr_offs, page_ent_val); //GICD
            }
            else if (target_addr >= 0x50042000 && target_addr < 0x50044000)
            {
                poke64(page_ent+arr_offs, page_ent_val + 0x4000); //GICC
            }
            /*else if (target_addr >= 0x700b0000 && target_addr < 0x700c0000)
            {
                poke64(page_ent+arr_offs, 0);
            }*/
            else if ((target_addr >= 0x7d000000 && target_addr < 0x7d006000) // USB
                     || (target_addr >= 0x70090000 && target_addr < 0x700a0000)
                     /*|| (target_addr >= 0x700d0000 && target_addr < 0x700da000)*/
                     /*|| (target_addr >= 0x70003000 && target_addr < 0x70004000) // pinmux
                     || (target_addr >= 0x6000d000 && target_addr < 0x6000E000)*/ // gpio 
                     || (target_addr >= 0x700b0000 && target_addr < 0x700c0000)
                     ) {
                poke64(page_ent+arr_offs, 0);
            }
            //else if (target_addr >= 0x01000000 && target_addr < 0x7d000000) //target_addr >= 0x700b0000 && target_addr < 0x700c0000
            //{
                //page_ent[i] = page_ent_val;
            //}
            else if (target_addr == 0x7001b000 || target_addr == 0x702ec000 || target_addr == 0x54300000) // HOS is mean.
            {
                poke64(page_ent+arr_offs, page_ent_val);
            }
            else
            {
                poke64(page_ent+arr_offs, page_ent_val);
                test = 2;
            }
            
            /*if (start_addr >= 0x00000000 && start_addr < 0x60000000) {
                poke64(page_ent+arr_offs, start_addr | or_val);
            }*/

            if (target_addr == 0x80060000) {
                println!("80060000 -> {:08x} {:x}", peek64(page_ent+arr_offs), test);
            }
        }
        
        dcache_flush(page_ent,0x1000);
        
        return page_ent | 0b11;
    }
}

pub fn vttbr_new_lv2_pagetable(start_addr: u64) -> u64
{
    unsafe
    {
        println!("Begin construct VTTBR lv2");
        let page_ent: u64 = vttbr_lv2_slab + (vttbr_lv2_slab_idx * 0x1000) as u64;
        vttbr_lv2_slab_idx += 1;

        println!("lv2 {:016x} for start_addr {:016x}", page_ent, start_addr);
        memset32(page_ent, 0, 0x1000);

        for i in 0..(0x1000/8)
        {
            let target_addr: u64 = (start_addr + (i * LV2_RANGE_SIZE) as u64);
            
            //poke64(page_ent + i*8, VTTBR_BLOCK_OR_VAL | target_addr);
            poke64(page_ent + i*8, vttbr_new_lv3_pagetable(target_addr));
        }
        
        dcache_flush(page_ent, 0x1000);
        
        return page_ent | 0b11;
    }
}

pub fn vttbr_construct()
{
    unsafe
    {
        println!("Begin construct VTTBR lv1");
        println!("lv1 {:016x}", vttbr_lv1);
        
        let entries = 8;
        for i in 0..entries
        {
            if (i <= 8) {
                poke64(vttbr_lv1 + i*8, vttbr_new_lv2_pagetable(i * LV1_RANGE_SIZE));//(i * LV1_RANGE_SIZE) | VTTBR_BLOCK_OR_VAL;
            }
            else if (i > 8) {
                poke64(vttbr_lv1 + i*8, 0);//vttbr_new_lv2_pagetable(i * LV1_RANGE_SIZE);
            }
        }
        
        dcache_flush(vttbr_lv1, 8*entries as usize);
        vttbr_apply(vttbr_lv1);
    }
}
