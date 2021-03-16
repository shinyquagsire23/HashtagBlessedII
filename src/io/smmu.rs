/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::util::*;

pub const AHB_BASE: u32 = 0x6000C000;

pub const AHB_ARBITRATION_DISABLE: u32 = (AHB_BASE + 0x004);

pub fn smmu_init()
{
    let ahb_arb_disable: MMIOReg = MMIOReg::new(AHB_ARBITRATION_DISABLE);
    
    // TODO actual init
    
    // Allow usbd regs to be arbitrated
    // (SMMU will still be locked out but there's a workaround)
    ahb_arb_disable.w32(0);
}
