/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use crate::arm::cache::*;
use crate::vm::funcs::*;
use crate::util::*;
use crate::arm::threading::get_core;
use core::mem;

const LV1_RANGE_SIZE: u64 = (0x040000000);
const LV2_RANGE_SIZE: u64 = (0x000200000);
const LV3_RANGE_SIZE: u64 = (0x000001000);

const VTTBR_BLOCK_OR_VAL:    u64 = (0x000000000004C5);
const VTTBR_PAGE_OR_VAL_IO:  u64 = (0x000000000004C7);
const VTTBR_PAGE_OR_VAL_MEM: u64 = (0x000000000004FF);
// SH[1:0] << 8
// S2AP[1:0] << 6
// MemAttr[3:0] << 2
// we want: write-back cacheable, non-sharable
// normal, gathering, reordering, early write ack

static mut VTTBR_LV2_SLAB_IDX: usize = 0;
static mut VTTBR_LV3_SLAB_IDX: usize = 0;

#[repr(align(0x1000))]
struct Lv1TTB([u64; 64]);

#[repr(align(0x1000))]
struct Lv2TTB([u64; 0x200*0x800]);

#[repr(align(0x1000))]
struct Lv3TTB([u64; 0x200*0x2000]);

static mut VTTBR_LV1: Lv1TTB = Lv1TTB([0; 64]);
static mut VTTBR_LV2_SLAB: Lv2TTB = Lv2TTB([0; 0x200*0x800]);
static mut VTTBR_LV3_SLAB: Lv3TTB = Lv3TTB([0; 0x200*0x2000]);

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
        println!("{:p} {:p} {:p}", &VTTBR_LV1, &VTTBR_LV2_SLAB, &VTTBR_LV3_SLAB);
    }
}

pub fn vttbr_new_lv3_pagetable(start_addr: u64) -> u64
{
    unsafe
    {
        let slice_start = VTTBR_LV3_SLAB_IDX * 0x200;
        let slice_end = slice_start + 0x200;
        let page_ent = VTTBR_LV3_SLAB.0.get_mut(slice_start..slice_end).unwrap();
        VTTBR_LV3_SLAB_IDX += 1;
        
        for i in 0..(0x1000/8) as usize
        {
            let mut target_addr: u64 = start_addr + ((i as u64) * LV3_RANGE_SIZE);
            target_addr = ipaddr_to_paddr(target_addr);

            let mut test = 0;
            let or_val = (if target_addr >= 0x80000000 { VTTBR_PAGE_OR_VAL_MEM } else { VTTBR_PAGE_OR_VAL_IO });
            let page_ent_val = or_val | target_addr;
            
            if (target_addr >= 0x80000000 && target_addr < 0x200000000) {
                page_ent[i] = page_ent_val;
            }
            /*else if (target_addr >= 0x50044000 && target_addr < 0x50046000)
            {
                page_ent[i] = 0;
                test = 1;
            }
            else if (target_addr >= 0x50040000 && target_addr < 0x50042000)
            {
                page_ent[i] = page_ent_val; //GICD
            }
            else if (target_addr >= 0x50042000 && target_addr < 0x50044000)
            {
                page_ent[i] = page_ent_val + 0x4000; //GICC
            }*/
            /*else if (target_addr >= 0x700b0000 && target_addr < 0x700c0000)
            {
                poke64(page_ent+arr_offs, 0);
            }*/
            else if ((target_addr >= 0x7d000000 && target_addr < 0x7d006000) // USB
                     || (target_addr >= 0x70090000 && target_addr < 0x700a0000)
                     /*|| (target_addr >= 0x700d0000 && target_addr < 0x700da000)*/
                     /*|| (target_addr >= 0x70003000 && target_addr < 0x70004000) // pinmux
                     || (target_addr >= 0x6000d000 && target_addr < 0x6000E000)*/ // gpio 
                     /*|| (target_addr >= 0x700b0000 && target_addr < 0x700c0000)*/  // sdmmc
                     ) {
                page_ent[i] = 0;
            }
            //else if (target_addr >= 0x01000000 && target_addr < 0x7d000000) //target_addr >= 0x700b0000 && target_addr < 0x700c0000
            //{
                //page_ent[i] = page_ent_val;
            //}
            else if (target_addr == 0x7001b000 || target_addr == 0x702ec000 || target_addr == 0x54300000) // HOS is mean.
            {
                page_ent[i] = page_ent_val;
            }
            else
            {
                page_ent[i] = page_ent_val;
                test = 2;
            }

            if (target_addr == 0x80060000) {
                println!("80060000 -> {:08x} {:x}", page_ent[i], test);
            }
        }
        
        dcache_flush(to_u64ptr!(page_ent.as_ptr()),0x1000);
        
        return to_u64ptr!(page_ent.as_ptr()) | 0b11;
    }
}

pub fn vttbr_new_lv2_pagetable(start_addr: u64) -> u64
{
    unsafe
    {
        println!("Begin construct VTTBR lv2");
        let slice_start = VTTBR_LV2_SLAB_IDX * 0x200;
        let slice_end = slice_start + 0x200;
        let page_ent = VTTBR_LV2_SLAB.0.get_mut(slice_start..slice_end).unwrap();
        VTTBR_LV2_SLAB_IDX += 1;

        println!("lv2 {:p} for start_addr {:016x}", page_ent.as_ptr(), start_addr);

        for i in 0..(0x1000/8)
        {
            let target_addr: u64 = (start_addr + ((i as u64) * LV2_RANGE_SIZE));
            
            //poke64(page_ent + i*8, VTTBR_BLOCK_OR_VAL | target_addr);
            page_ent[i] = vttbr_new_lv3_pagetable(target_addr);
        }
        
        dcache_flush(to_u64ptr!(page_ent.as_ptr()), 0x1000);
        
        return to_u64ptr!(page_ent.as_ptr()) | 0b11;
    }
}

pub fn vttbr_construct()
{
    unsafe
    {
        println!("Begin construct VTTBR lv1");
        println!("lv1 {:p} {:x}", VTTBR_LV1.0.as_ptr(), mem::size_of::<Lv1TTB>());

        let entries = 32;
        for i in 0..entries
        {
            if (i <= 8) {
                VTTBR_LV1.0[i] = vttbr_new_lv2_pagetable(i as u64 * LV1_RANGE_SIZE);//(i * LV1_RANGE_SIZE) | VTTBR_BLOCK_OR_VAL;
            }
            else if (i > 8) {
                VTTBR_LV1.0[i] = 0; // vttbr_new_lv2_pagetable(i * LV1_RANGE_SIZE)
            }
        }
        
        dcache_flush(to_u64ptr!(VTTBR_LV1.0.as_ptr()), mem::size_of::<Lv1TTB>());
        vttbr_apply(VTTBR_LV1.0.as_ptr());
    }
}

pub fn vttbr_transfer_newcore()
{
    unsafe
    {
        // TODO get these sizes dynamically?
        dcache_invalidate(to_u64ptr!(VTTBR_LV1.0.as_ptr()), mem::size_of::<Lv1TTB>());
        dcache_invalidate(to_u64ptr!(VTTBR_LV2_SLAB.0.as_ptr()), mem::size_of::<Lv2TTB>());
        dcache_invalidate(to_u64ptr!(VTTBR_LV3_SLAB.0.as_ptr()), mem::size_of::<Lv3TTB>());
        vttbr_apply(VTTBR_LV1.0.as_ptr());
    }
}
