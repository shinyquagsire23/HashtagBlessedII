/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("cache.s"));

extern "C" {
    pub fn dcache_flush_invalidate(addr: i64, size: i64);
    pub fn dcache_invalidate(addr: i64, size: i64);
    pub fn icache_invalidate(addr: i64, size: i64);
    pub fn dcache_flush(addr: i64, size: i64);
    pub fn dcache_zero(addr: i64, size: i64);

}
