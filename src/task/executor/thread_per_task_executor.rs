/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::thread;

use qubit_function::Callable;

use crate::task::{
    TaskHandle,
    task_runner::run_callable,
};

use super::Executor;

/// Executes each task on a dedicated OS thread.
///
/// This executor does not manage lifecycle or maintain a queue. Each accepted
/// task receives a blocking [`TaskHandle`] that can be used to wait for the
/// result.
///
/// # Semantics
///
/// * **One task, one thread** — each [`Executor::call`] or [`Executor::execute`]
///   spawns a new [`std::thread::spawn`] worker. There is no pool and no
///   submission queue.
/// * **Blocking wait** — [`TaskHandle::get`] performs a blocking
///   [`std::sync::mpsc::Receiver::recv`] on the calling thread. Do not call it
///   from a Tokio async worker thread unless you offload with
///   [`tokio::task::spawn_blocking`] or similar.
/// * **Completion probe** — [`TaskHandle::is_done`] reads an atomic flag set
///   after the worker sends the result; it does not retrieve the value (you
///   still need [`TaskHandle::get`] for that).
///
/// # Examples
///
/// ```rust
/// use std::io;
///
/// use qubit_concurrent::task::executor::{
///     Executor,
///     ThreadPerTaskExecutor,
/// };
///
/// let executor = ThreadPerTaskExecutor;
/// let handle = executor.call(|| Ok::<i32, io::Error>(40 + 2));
///
/// // Blocks the current thread until the spawned thread completes.
/// let value = handle.get().expect("task should succeed");
/// assert_eq!(value, 42);
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadPerTaskExecutor;

impl Executor for ThreadPerTaskExecutor {
    type Execution<R, E>
        = TaskHandle<R, E>
    where
        R: Send + 'static,
        E: Send + 'static;

    /// Spawns one OS thread for the callable and returns a handle to its result.
    fn call<C, R, E>(&self, task: C) -> Self::Execution<R, E>
    where
        C: Callable<R, E> + Send + 'static,
        R: Send + 'static,
        E: Send + 'static,
    {
        let (handle, sender, done) = TaskHandle::channel();
        thread::spawn(move || {
            let result = run_callable(task);
            let _ = sender.send(result);
            done.store(true);
        });
        handle
    }
}
