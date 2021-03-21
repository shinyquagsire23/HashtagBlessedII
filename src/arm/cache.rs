/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

global_asm!(include_str!("cache.s"));

extern "C" {
    pub fn _dcache_flush_invalidate(addr: u64, size: usize);
    pub fn _dcache_invalidate(addr: u64, size: usize);
    pub fn _icache_invalidate(addr: u64, size: usize);
    pub fn _dcache_flush(addr: u64, size: usize);
    pub fn _dcache_zero(addr: u64, size: usize);

}

pub fn dcache_flush_invalidate(addr: u64, size: usize)
{
    unsafe { _dcache_flush_invalidate(addr, size); }
}

pub fn dcache_invalidate(addr: u64, size: usize)
{
    unsafe { _dcache_invalidate(addr, size); }
}

pub fn icache_invalidate(addr: u64, size: usize)
{
    unsafe { _icache_invalidate(addr, size); }
}

pub fn dcache_flush(addr: u64, size: usize)
{
    unsafe { _dcache_flush(addr, size); }
}

pub fn dcache_zero(addr: u64, size: usize)
{
    unsafe { _dcache_zero(addr, size); }
}
