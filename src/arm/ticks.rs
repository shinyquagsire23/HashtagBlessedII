/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

pub fn get_ticks() -> u64
{
    sysreg_read!("cntpct_el0")
}

pub fn get_tick_freq() -> u64
{
    sysreg_read!("cntfrq_el0")
}

pub fn ns_to_ticks(ns: u64) -> u64
{
    return (ns * 12) / 625;
}

pub fn ticks_to_ns(ticks: u64) -> u64
{
    return (ticks * 625) / 12;
}

pub fn us_to_ns(secs: u64) -> u64
{
    secs * 1000
}

pub fn ms_to_ns(secs: u64) -> u64
{
    secs * 1000000
}

pub fn secs_to_ns(secs: u64) -> u64
{
    secs * 1000000000
}
