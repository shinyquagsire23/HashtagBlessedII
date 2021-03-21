/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use crate::arm::ticks::*;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use crate::logger::*;

pub struct SleepNs 
{
    wake_cnt: u64
}

impl Future for SleepNs 
{
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> 
    {
        if self.wake_cnt <= get_ticks()
        {
            Poll::Ready(())
        } 
        else 
        {
            Poll::Pending
        }
    }
}

impl SleepNs
{
    pub fn new(ns: u64) -> Self {
        SleepNs { wake_cnt: get_ticks() + ns_to_ticks(ns) }
    }
}
