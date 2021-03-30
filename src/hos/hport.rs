/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::cmp::{Ordering, Eq};
use crate::vm::vsvc::vsvc_get_curpid;
use alloc::string::String;
use super::hclientsession::HClientSession;

pub struct HPort
{
    pub pid: u8,
    pub name: Option<String>
}

impl HPort
{
    pub fn from_curpid(name: Option<String>) -> Self
    {
        HPort
        {
            pid: (vsvc_get_curpid() & 0xFF) as u8,
            name: name
        }
    }
    
    pub fn create_session(&self) -> HClientSession
    {
        HClientSession::new(self.pid, (vsvc_get_curpid() & 0xFF) as u8)
    }
}
