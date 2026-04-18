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
};

use tokio::{sync::Notify, task::JoinHandle};

use super::{async_executor::AsyncExecutor, async_executor_service::AsyncExecutorService};

/// Shared state for `TokioExecutorService`.
#[derive(Default)]
struct TokioExecutorServiceState {
    shutdown: AtomicBool,
    active_tasks: AtomicUsize,
    handles: Mutex<Vec<JoinHandle<()>>>,
    terminated_notify: Notify,
}

impl TokioExecutorServiceState {
    /// Acquires the task handle collection while tolerating poisoned locks.
    fn lock_handles(&self) -> MutexGuard<'_, Vec<JoinHandle<()>>> {
        self.handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Drops handles for tasks that have already completed.
    fn cleanup_finished_handles(&self) {
        self.lock_handles().retain(|handle| !handle.is_finished());
    }

    /// Notifies awaiters if shutdown has been requested and no tasks remain.
    fn notify_if_terminated(&self) {
        if self.shutdown.load(Ordering::Acquire) && self.active_tasks.load(Ordering::Acquire) == 0 {
            self.terminated_notify.notify_waiters();
        }
    }
}

/// Task lifecycle guard for `TokioExecutorService`.
///
/// Decrements active task count and signals termination on drop.
struct TokioTaskGuard {
    state: Arc<TokioExecutorServiceState>,
}

impl TokioTaskGuard {
    /// Creates a new lifecycle guard.
    #[inline]
    fn new(state: Arc<TokioExecutorServiceState>) -> Self {
        Self { state }
    }
}

impl Drop for TokioTaskGuard {
    /// Updates service counters when a task completes or is cancelled.
    fn drop(&mut self) {
        if self.state.active_tasks.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.state.notify_if_terminated();
        }
    }
}

/// Tokio-backed async executor with lifecycle control.
///
/// After shutdown, new tasks are ignored. `shutdown_now()` aborts tracked tasks.
#[derive(Default, Clone)]
pub struct TokioExecutorService {
    state: Arc<TokioExecutorServiceState>,
}

impl TokioExecutorService {
    /// Creates a new service instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl AsyncExecutor for TokioExecutorService {
    /// Spawns an async task while the service is accepting work.
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.state.shutdown.load(Ordering::Acquire) {
            return;
        }
        self.state.active_tasks.fetch_add(1, Ordering::AcqRel);
        let state = Arc::clone(&self.state);
        let handle = tokio::spawn(async move {
            let _guard = TokioTaskGuard::new(state);
            future.await;
        });
        self.state.cleanup_finished_handles();
        self.state.lock_handles().push(handle);
    }
}

impl AsyncExecutorService for TokioExecutorService {
    /// Stops accepting new async tasks.
    fn shutdown(&self) {
        self.state.shutdown.store(true, Ordering::Release);
        self.state.notify_if_terminated();
    }

    /// Stops accepting new tasks and aborts tracked Tokio tasks.
    fn shutdown_now(&self) {
        self.state.shutdown.store(true, Ordering::Release);
        let mut handles = self.state.lock_handles();
        let draining = std::mem::take(&mut *handles);
        drop(handles);
        for handle in draining {
            handle.abort();
        }
        self.state.notify_if_terminated();
    }

    /// Returns whether shutdown has been requested.
    fn is_shutdown(&self) -> bool {
        self.state.shutdown.load(Ordering::Acquire)
    }

    /// Returns whether shutdown was requested and no active tasks remain.
    fn is_terminated(&self) -> bool {
        self.is_shutdown() && self.state.active_tasks.load(Ordering::Acquire) == 0
    }

    /// Waits until shutdown is requested and all active tasks complete.
    fn await_termination(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            loop {
                self.state.cleanup_finished_handles();
                if self.is_terminated() {
                    return;
                }
                self.state.terminated_notify.notified().await;
            }
        })
    }
}
