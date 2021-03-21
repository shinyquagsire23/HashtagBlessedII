/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor 
{
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    task_queue_next: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor 
{
    pub fn new() -> Self 
    {
        Executor 
        {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            task_queue_next: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn queue(&mut self, task: Task) 
    {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some()
        {
            panic!("task with ID {} already exists in task queue", task_id);
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    pub fn advance(&mut self)
    {
        self.run_ready_tasks();
    }

    fn run_ready_tasks(&mut self) 
    {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            task_queue_next,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() 
        {
            let task = match tasks.get_mut(&task_id) 
            {
                Some(task) => task,
                None => continue, // Task no longer exists
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) 
            {
                Poll::Ready(()) => 
                {
                    // Task is done, remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => 
                {
                    // Continue to poll these tasks on the next advance
                    task_queue_next.push(task_id).expect("queue full");
                }
            }
        }
        
        // Move remaining tasks to task queue
        while let Ok(task_id) = task_queue_next.pop() 
        {
            task_queue.push(task_id).expect("queue full");
        }
    }

    fn sleep_if_idle(&self)
    {
        // disable interrupts
        if self.task_queue.is_empty()
        {
            // TODO enable interrupts and WFI?
        } 
        else
        {
            // enable interrupts
        }
    }
}

struct TaskWaker 
{
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker 
{
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker 
    {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) 
    {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker 
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
