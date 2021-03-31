/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::vm::vsvc::vsvc_get_svc_addr;
use crate::task::svc_wait::SvcWait;

pub async fn hsvc_sleep_thread(mut pre_ctx: [u64; 32], ns: u64) -> [u64; 32]
{
    let svc_handler = vsvc_get_svc_addr(0x0B);
    
    let mut sleep_ctx = pre_ctx.clone();
    sleep_ctx[0] = ns;
    
    //println!("{:x} {:x}", svc_handler, sleep_ctx[11]);
    
    sleep_ctx[30] = sleep_ctx[31]+4;
    sleep_ctx[31] = svc_handler;
    
    // Wait for SVC to complete
    let post_ctx = SvcWait::new(sleep_ctx).await;
    
    // Process SVC output
    
    // Restore last context
    return pre_ctx;
}

pub async fn hsvc_sleep_thread_2(mut pre_ctx: [u64; 32], ns: u64) -> [u64; 32]
{
    let svc_handler = vsvc_get_svc_addr(0x0B);
    
    let mut sleep_ctx = pre_ctx.clone();
    sleep_ctx[0] = ns;
    
    //println!("{:x} {:x}", svc_handler, sleep_ctx[11]);
    
    sleep_ctx[31] = svc_handler;
    
    // Wait for SVC to complete
    let post_ctx = SvcWait::new(sleep_ctx).await;
    
    // Process SVC output
    
    // Restore last context
    return post_ctx;
}
