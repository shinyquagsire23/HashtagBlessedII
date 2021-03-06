/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use core::future::Future;
use alloc::prelude::v1::Box;
use core::pin::Pin;
use super::hdomainobj::HDomainObj;
use super::hdomainsession::HDomainSession;
use super::hipc::{HExtraNone, HObject, HObjectExtra};

pub type HClientSessionHandler = fn(in_: [u64; 32], hobj: HObject) -> Pin<Box<dyn Future<Output = [u64; 32]> + Send>>;
 
pub struct HClientSession
{
    pub parent_port_pid: u8,
    pub client_pid: u8,
    pub handler: Option<HClientSessionHandler>,
    pub extra: HObjectExtra
}

impl HClientSession
{
    pub fn new(parent_pid: u8, client_pid: u8) -> Self
    {
        HClientSession
        {
            parent_port_pid: parent_pid,
            client_pid: client_pid,
            handler: None,
            extra: HObjectExtra::None(HExtraNone{})
        }
    }
    
    pub fn new_from_parent(&self) -> Self
    {
        HClientSession
        {
            parent_port_pid: self.parent_port_pid,
            client_pid: self.client_pid,
            handler: None,
            extra: self.extra.clone()
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
    
    pub fn set_extra(&mut self, extra: HObjectExtra)
    {
        self.extra = extra;
    }
    
    pub fn get_extra(&self) -> HObjectExtra
    {
        self.extra.clone()
    }
    
    pub fn convert_to_domain(&self, handle: u32, obj_id: u32) -> (HDomainObj, HDomainSession)
    {
        let mut obj = HDomainObj::from_curpid(handle, obj_id);
        let mut session = HDomainSession::new(self.parent_port_pid, self.client_pid);
        
        if let Some(handler) = self.handler
        {
            session.set_handler(handler);
        }
        return (obj, session);
    }
    
    pub fn clone(&self) -> Self
    {
        HClientSession
        {
            parent_port_pid: self.parent_port_pid,
            client_pid: self.client_pid,
            handler: self.handler,
            extra: self.extra.clone()
        }
    }
}
