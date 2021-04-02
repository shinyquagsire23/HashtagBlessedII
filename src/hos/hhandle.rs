/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::cmp::{Ordering, Eq};
use crate::vm::vsvc::vsvc_get_curpid;

pub struct HHandle
{
    pub pid: u8,
    pub handle: u32
}

impl PartialOrd for HHandle 
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> 
    {
        Some(self.cmp(other))
    }
}

impl Ord for HHandle 
{
    fn cmp(&self, other: &Self) -> Ordering 
    {
        if self.pid == other.pid
        {
            return self.handle.cmp(&other.handle)
        }
        else
        {
            return self.pid.cmp(&other.pid);
        }
    }
}

impl PartialEq for HHandle 
{
    fn eq(&self, other: &Self) -> bool 
    {
        self.pid == other.pid && self.handle == other.handle
    }
}
impl Eq for HHandle {}

impl HHandle
{
    pub fn from_curpid(handle: u32) -> Self
    {
        HHandle
        {
            pid: (vsvc_get_curpid() & 0xFF) as u8,
            handle: handle
        }
    }
}
