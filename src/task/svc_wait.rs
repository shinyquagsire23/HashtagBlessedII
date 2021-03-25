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
use crate::arm::threading::*;

static mut SVCWAIT_CTX: [[u64; 32]; 4] = [[0; 32]; 4];

pub struct SvcWait 
{
    polled_ticks: u8
}

impl Future for SvcWait 
{
    type Output = [u64; 32];
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> 
    {
        if self.polled_ticks >= 1
        {
            unsafe
            {
                Poll::Ready(SVCWAIT_CTX[get_core() as usize])
            }
        } 
        else 
        {
            self.polled_ticks += 1;
            Poll::Pending
        }
    }
}

impl SvcWait
{
    pub fn populate_ctx(ctx: [u64; 32]) {
        unsafe
        {
            SVCWAIT_CTX[get_core() as usize] = ctx;
        }
    }
    
    pub fn get_ctx() -> [u64; 32] {
        unsafe
        {
            return SVCWAIT_CTX[get_core() as usize];
        }
    }
    
    pub fn new(ctx: [u64; 32]) -> Self {
        unsafe
        {
            SVCWAIT_CTX[get_core() as usize] = ctx;
        }
        SvcWait { polled_ticks: 0 }
    }
}
