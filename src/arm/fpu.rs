/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

pub fn fpu_enable()
{
    sysreg_or64!("cpacr_el1", (3 << 20));
}
