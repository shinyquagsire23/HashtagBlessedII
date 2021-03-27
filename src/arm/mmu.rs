/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

pub const TTB_LV1_MASK: u64 = 0x7FC0000000;
pub const TTB_LV1_SHIFT_64KIB: u64 = 29;
pub const TTB_LV1_SHIFT: u64 = 30;
pub const TTB_LV1_ADD:   u64 = 0x40000000;
pub const TTB_LV2_MASK:  u64 = 0x3FE00000;
pub const TTB_LV2_SHIFT: u64 = 21;
pub const TTB_LV2_ADD:   u64 = 0x200000;
pub const TTB_LV3_MASK:  u64 = 0x1FF000;
pub const TTB_LV3_SHIFT: u64 = 12;
pub const TTB_LV3_ADD:   u64 = 0x1000;

pub const TTB_LV12_OR:      u64 = 0x3000000000000003 ;
pub const TTB_IO_LV12_OR:   u64 = 0x3800000000000003;
pub const TTB_MEM_LV3_OR:   u64 = 0x40000000000003 | (0x102 << 2); /*id 2, AF*/
pub const TTB_IO_LV3_OR:    u64 = 0x60000000000604 | 3;
pub const TTB_LV_ADDR_MASK: u64 = 0xFFFFF000;

pub const TTB_AP_SHIFT:   u64 = 6;
pub const TTB_AP_UNO_KRW: u64 = 0;
pub const TTB_AP_URW_KRW: u64 = 1;
pub const TTB_AP_UNO_KRO: u64 = 2;
pub const TTB_AP_URO_KRO: u64 = 3;

pub fn get_ttbr1_el1() -> u64
{
    sysreg_read!("ttbr1_el1")
}

pub fn translate_el1_stage12(vaddr: u64) -> u64
{
    unsafe
    {
    let mut taddr: u64 = 0;
    asm!("AT S12E1R, {0}", in(reg) vaddr);
    asm!("mrs {0}, PAR_EL1", out(reg) taddr);

    return (taddr & 0xffffffffff000) | (vaddr & 0xFFF);
    }
}

pub fn translate_el0_stage12(vaddr: u64) -> u64
{
    unsafe
    {
    let mut taddr: u64 = 0;
    asm!("AT S12E0R, {0}", in(reg) vaddr);
    asm!("mrs {0}, PAR_EL1", out(reg) taddr);

    return (taddr & 0xffffffffff000) | (vaddr & 0xFFF);
    }
}

pub fn translate_el1_stage1(vaddr: u64) -> u64
{
    unsafe
    {
    let mut taddr: u64 = 0;
    asm!("AT S1E1R, {0}", in(reg) vaddr);
    asm!("mrs {0}, PAR_EL1", out(reg) taddr);

    return (taddr & 0xffffffffff000) | (vaddr & 0xFFF);
    }
}
