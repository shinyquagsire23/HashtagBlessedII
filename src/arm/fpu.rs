/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("fpu.s"));

extern "C" {
    pub fn enable_fp();
}

pub fn fpu_enable()
{
    unsafe
    {
        enable_fp();
    }
}
