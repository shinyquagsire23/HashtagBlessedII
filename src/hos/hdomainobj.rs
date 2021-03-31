/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::cmp::{Ordering, Eq};
use crate::vm::vsvc::vsvc_get_curpid;

pub struct HDomainObj
{
    pid: u8,
    handle: u32,
    id: u32
}

impl PartialOrd for HDomainObj 
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> 
    {
        Some(self.cmp(other))
    }
}

impl Ord for HDomainObj 
{
    fn cmp(&self, other: &Self) -> Ordering 
    {
        if self.pid == other.pid
        {
            if self.handle == other.handle
            {
                return self.id.cmp(&other.id)
            }
            return self.handle.cmp(&other.handle)
        }
        else
        {
            return self.pid.cmp(&other.pid);
        }
    }
}

impl PartialEq for HDomainObj 
{
    fn eq(&self, other: &Self) -> bool 
    {
        self.pid == other.pid && self.handle == other.handle && self.id == other.id
    }
}
impl Eq for HDomainObj {}

impl HDomainObj
{
    pub fn from_curpid(handle: u32, id: u32) -> Self
    {
        HDomainObj
        {
            pid: (vsvc_get_curpid() & 0xFF) as u8,
            handle: handle,
            id: id
        }
    }
}
