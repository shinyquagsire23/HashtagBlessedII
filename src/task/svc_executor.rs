/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};
use derive_more::Display;
use alloc::{collections::BTreeMap, sync::Arc, task::Wake, boxed::Box};
use crossbeam_queue::ArrayQueue;
use core::option::Option;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub struct SvcTaskId(pub u64);

impl SvcTaskId 
{
    fn new() -> Self 
    {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        SvcTaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct SvcTask
{
    pub id: SvcTaskId,
    future: Pin<Box<dyn Future<Output = ([u64; 32])>>>,
}

impl SvcTask 
{
    pub fn new(thread_ctx: u64, future: impl Future<Output = ([u64; 32])> + 'static) -> SvcTask 
    {
        SvcTask 
        {
            id: SvcTaskId(thread_ctx),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<([u64; 32])> 
    {
        self.future.as_mut().poll(context)
    }
}

pub struct SvcExecutor 
{
    tasks: BTreeMap<SvcTaskId, SvcTask>,
    waker_cache: BTreeMap<SvcTaskId, Waker>,
}

impl SvcExecutor 
{
    pub fn new() -> Self 
    {
        SvcExecutor 
        {
            tasks: BTreeMap::new(),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn queue(&mut self, task: SvcTask) 
    {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some()
        {
            //panic!("task with ID {} already exists in task queue", task_id);
        }
    }

    pub fn run_svc(&mut self, task_id: SvcTaskId) -> Option<[u64; 32]>
    {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            waker_cache,
        } = self;

        let task = match tasks.get_mut(&task_id) 
        {
            Some(task) => task,
            None => return None, // Task no longer exists
        };

        let waker = waker_cache
            .entry(task_id)
            .or_insert_with(|| SvcTaskWaker::new(task_id));

        let mut context = Context::from_waker(waker);
        match task.poll(&mut context) 
        {
            Poll::Ready(ctx_result) => 
            {
                // Task is done, remove it and its cached waker
                tasks.remove(&task_id);
                waker_cache.remove(&task_id);
                
                return Some(ctx_result);
            }
            Poll::Pending => 
            {
                return None
            }
        }
    }
}

struct SvcTaskWaker 
{
    task_id: SvcTaskId,
}

impl SvcTaskWaker 
{
    fn new(task_id: SvcTaskId) -> Waker 
    {
        Waker::from(Arc::new(SvcTaskWaker {
            task_id,
        }))
    }

    fn wake_task(&self) 
    {
        //self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for SvcTaskWaker 
{
    fn wake(self: Arc<Self>)
    {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>)
    {
        self.wake_task();
    }
}
