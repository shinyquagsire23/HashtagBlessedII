/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
pub struct HClientSession
{
    pub parent_port_pid: u8,
    pub client_pid: u8,
}

impl HClientSession
{
    pub fn new(parent_pid: u8, client_pid: u8) -> Self
    {
        HClientSession
        {
            parent_port_pid: parent_pid,
            client_pid: client_pid
        }
    }
}
