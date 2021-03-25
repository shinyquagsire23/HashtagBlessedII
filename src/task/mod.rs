/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};
use derive_more::Display;
use executor::Executor;
use svc_executor::{SvcExecutor, SvcTask, SvcTaskId};

pub mod executor;
pub mod sleep;
pub mod svc_executor;
pub mod svc_wait;

pub struct Task
{
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task 
{
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task 
    {
        Task 
        {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> 
    {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
struct TaskId(u64);

impl TaskId 
{
    fn new() -> Self 
    {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

static mut EXECUTOR: Option<Executor> = None;
static mut SVC_EXECUTOR: Option<SvcExecutor> = None;

pub fn task_init()
{
    unsafe
    {
        EXECUTOR = Some(Executor::new());
        SVC_EXECUTOR = Some(SvcExecutor::new());
    }
}

//TODO multi-core safety
pub fn task_run(future: impl Future<Output = ()> + 'static)
{
    unsafe
    {
        let mut executor = EXECUTOR.as_mut().unwrap();
        executor.queue(Task::new(future));
    }
}

pub fn task_queue(task: Task)
{
    unsafe
    {
        let mut executor = EXECUTOR.as_mut().unwrap();
        executor.queue(task);
    }
}

pub fn task_advance()
{
    unsafe
    {
        let mut executor = EXECUTOR.as_mut().unwrap();
        executor.advance();
    }
}

pub fn task_advance_svc(task_id: SvcTaskId) -> Option<[u64; 32]>
{
    unsafe
    {
        let mut executor = SVC_EXECUTOR.as_mut().unwrap();
        executor.run_svc(task_id)
    }
}

pub fn task_advance_svc_ctx(thread_ctx: u64) -> Option<[u64; 32]>
{
    unsafe
    {
        let mut executor = SVC_EXECUTOR.as_mut().unwrap();
        executor.run_svc(SvcTaskId(thread_ctx))
    }
}

pub fn task_run_svc(thread_ctx: u64, future: impl Future<Output = ([u64; 32])> + 'static) -> SvcTaskId
{
    unsafe
    {
        let mut executor = SVC_EXECUTOR.as_mut().unwrap();
        let task = SvcTask::new(thread_ctx, future);
        let task_id = task.id;
        executor.queue(task);
        return task_id;
    }
}
