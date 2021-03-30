/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use core::future::Future;
use alloc::prelude::v1::Box;
use core::pin::Pin;

pub type HClientSessionHandler = fn(in_: [u64; 32]) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>>;
 
pub struct HClientSession
{
    pub parent_port_pid: u8,
    pub client_pid: u8,
    pub handler: Option<HClientSessionHandler>
}

impl HClientSession
{
    pub fn new(parent_pid: u8, client_pid: u8) -> Self
    {
        HClientSession
        {
            parent_port_pid: parent_pid,
            client_pid: client_pid,
            handler: None
        }
    }
    
    pub fn set_handler(&mut self, handler: HClientSessionHandler)
    {
        self.handler = Some(handler);
    }
    
    pub fn get_handler(&self) -> Option<HClientSessionHandler>
    {
        self.handler
    }
    
    pub fn clone(&self) -> Self
    {
        HClientSession
        {
            parent_port_pid: self.parent_port_pid,
            client_pid: self.client_pid,
            handler: self.handler
        }
    }
}
