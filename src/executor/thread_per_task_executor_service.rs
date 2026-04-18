/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread,
};

use super::{executor::Executor, executor_service::ExecutorService, runnable::Runnable};

/// Shared state for `ThreadPerTaskExecutorService`.
#[derive(Default)]
struct ThreadPerTaskExecutorServiceState {
    shutdown: AtomicBool,
    active_tasks: AtomicUsize,
    handles: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl ThreadPerTaskExecutorServiceState {
    /// Acquires the thread handle collection while tolerating poisoned locks.
    fn lock_handles(&self) -> MutexGuard<'_, Vec<thread::JoinHandle<()>>> {
        self.handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Joins all currently tracked worker threads.
    ///
    /// This method blocks until every tracked thread has completed.
    fn join_all_handles(&self) {
        let mut handles = self.lock_handles();
        let running = std::mem::take(&mut *handles);
        drop(handles);
        for handle in running {
            let _ = handle.join();
        }
    }
}

/// Thread-per-task executor with shutdown and termination coordination.
///
/// This type offers a simple `ExecutorService` implementation for synchronous
/// tasks. `shutdown_now()` cannot forcefully stop running threads; it only
/// stops accepting new tasks.
#[derive(Default, Clone)]
pub struct ThreadPerTaskExecutorService {
    state: Arc<ThreadPerTaskExecutorServiceState>,
}

impl ThreadPerTaskExecutorService {
    /// Creates a new service instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Executor for ThreadPerTaskExecutorService {
    /// Spawns a dedicated worker thread for the task when the service is active.
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        if self.state.shutdown.load(Ordering::Acquire) {
            return;
        }
        self.state.active_tasks.fetch_add(1, Ordering::AcqRel);
        let state = Arc::clone(&self.state);
        let handle = thread::spawn(move || {
            task();
            state.active_tasks.fetch_sub(1, Ordering::AcqRel);
        });
        self.state.lock_handles().push(handle);
    }
}

impl ExecutorService for ThreadPerTaskExecutorService {
    /// Stops accepting new synchronous tasks.
    fn shutdown(&self) {
        self.state.shutdown.store(true, Ordering::Release);
    }

    /// Stops accepting new synchronous tasks and returns no queued tasks.
    ///
    /// The service executes submitted work immediately, so there is no pending
    /// task queue to return.
    fn shutdown_now(&self) -> Vec<Box<dyn Runnable>> {
        self.state.shutdown.store(true, Ordering::Release);
        Vec::new()
    }

    /// Returns whether shutdown has been requested.
    fn is_shutdown(&self) -> bool {
        self.state.shutdown.load(Ordering::Acquire)
    }

    /// Returns whether shutdown was requested and all tasks are finished.
    fn is_terminated(&self) -> bool {
        self.is_shutdown() && self.state.active_tasks.load(Ordering::Acquire) == 0
    }

    /// Waits for all tracked worker threads to complete.
    ///
    /// This future blocks the current thread while joining worker threads.
    fn await_termination(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.state.join_all_handles();
        })
    }
}
