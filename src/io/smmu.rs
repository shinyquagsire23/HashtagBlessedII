/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::util::*;

pub const AHB_BASE: u32 = 0x6000C000;

pub const AHB_ARBITRATION_DISABLE: u32 = (AHB_BASE + 0x004);

pub const MC_BASE: u64 = (0x70019000);
pub const MC_END: u64 = (0x7001A000);

pub const MC_SMMU_CONFIG: u64 = (MC_BASE + 0x10);
pub const MC_SMMU_TLB_CONFIG: u64 = (MC_BASE + 0x14);
pub const MC_SMMU_PTC_CONFIG: u64 = (MC_BASE + 0x18);
pub const MC_SMMU_PTB_ASID: u64 = (MC_BASE + 0x1C);
pub const MC_SMMU_PTB_DATA: u64 = (MC_BASE + 0x20);
pub const MC_SMMU_TLB_FLUSH: u64 = (MC_BASE + 0x30);
pub const MC_SMMU_PTC_FLUSH: u64 = (MC_BASE + 0x34);
pub const MC_SMMU_PPCS1_ASID: u64 = (MC_BASE + 0x298);
pub const MC_SMMU_SDMMC2A_ASID: u64 = (MC_BASE + 0xA98);
pub const MC_SMMU_SDMMC3A_ASID: u64 = (MC_BASE + 0xA9C);

pub fn smmu_init()
{
    let ahb_arb_disable: MMIOReg = MMIOReg::new(AHB_ARBITRATION_DISABLE);
    
    // TODO actual init
    
    // Allow usbd regs to be arbitrated
    // (SMMU will still be locked out but there's a workaround)
    ahb_arb_disable.w32(0);
}
