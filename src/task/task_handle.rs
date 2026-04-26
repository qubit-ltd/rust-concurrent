/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::{
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    task::{Context, Poll, Waker},
};

use crate::lock::Monitor;

use super::{TaskExecutionError, TaskResult};

/// Handle for a task running outside the caller's current stack.
///
/// `TaskHandle` is returned by thread-backed executors and services. Calling
/// [`Self::get`] blocks the current thread until the accepted task completes.
/// Awaiting the handle waits asynchronously for the same final task result.
///
/// # Type Parameters
///
/// * `R` - The task success value.
/// * `E` - The task error value.
///
/// # Author
///
/// Haixing Hu
pub struct TaskHandle<R, E> {
    /// Shared state observed by the handle and updated by completion endpoints.
    inner: Arc<TaskHandleInner<R, E>>,
}

/// Shared state used by a task handle and its completing runner.
struct TaskHandleInner<R, E> {
    /// Final task result written once by completion and read once by handle.
    result: UnsafeCell<Option<TaskResult<R, E>>>,
    /// Monitor protecting blocking waiters and async waker.
    wait_state: Monitor<TaskHandleWaitState>,
    /// Atomic lifecycle for cheap start, cancel, and completion probes.
    status: AtomicU8,
}

/// Mutable waiter state protected by the task handle monitor.
struct TaskHandleWaitState {
    /// Number of threads currently blocked in [`TaskHandle::get`].
    blocking_waiters: usize,
    /// Last async waker registered by polling the handle before completion.
    waker: Option<Waker>,
}

/// Completion endpoint owned by a task runner.
///
/// This low-level endpoint is exposed so custom executor services built on top
/// of `qubit-concurrent` can wire their own scheduling and cancellation hooks
/// while still returning the standard [`TaskHandle`]. Normal callers should use
/// [`TaskHandle`] and executor/service submission methods instead.
pub struct TaskCompletion<R, E> {
    /// Shared state updated by this completion endpoint.
    inner: Arc<TaskHandleInner<R, E>>,
}

/// Task has been accepted but has not started.
const TASK_PENDING: u8 = 0;
/// Task runner has started executing the callable.
const TASK_RUNNING: u8 = 1;
/// A completion path is publishing the final result.
const TASK_PUBLISHING: u8 = 2;
/// Final result has been published.
const TASK_COMPLETED: u8 = 3;

unsafe impl<R: Send, E: Send> Send for TaskHandleInner<R, E> {}
unsafe impl<R: Send, E: Send> Sync for TaskHandleInner<R, E> {}

impl<R, E> TaskHandle<R, E> {
    /// Creates a handle and completion endpoint used by a task runner.
    ///
    /// # Returns
    ///
    /// A handle for the caller and a completion endpoint for the runner.
    pub fn completion_pair() -> (Self, TaskCompletion<R, E>) {
        let inner = Arc::new(TaskHandleInner {
            result: UnsafeCell::new(None),
            wait_state: Monitor::new(TaskHandleWaitState {
                blocking_waiters: 0,
                waker: None,
            }),
            status: AtomicU8::new(TASK_PENDING),
        });
        let handle = Self {
            inner: Arc::clone(&inner),
        };
        let completion = TaskCompletion { inner };
        (handle, completion)
    }

    /// Waits for the task to finish and returns its final result.
    ///
    /// This method blocks the current thread until a result is available.
    ///
    /// # Returns
    ///
    /// `Ok(R)` if the task succeeds. If the accepted task returns `Err(E)`,
    /// panics, or is cancelled before producing a value, the corresponding
    /// [`TaskExecutionError`] is returned.
    pub fn get(self) -> TaskResult<R, E> {
        if self.inner.status.load(Ordering::Acquire) != TASK_COMPLETED {
            let mut wait_state = self.inner.wait_state.lock();
            if self.inner.status.load(Ordering::Acquire) != TASK_COMPLETED {
                wait_state.blocking_waiters += 1;
                while self.inner.status.load(Ordering::Acquire) != TASK_COMPLETED {
                    wait_state = wait_state.wait();
                }
                wait_state.blocking_waiters = wait_state
                    .blocking_waiters
                    .checked_sub(1)
                    .expect("task handle blocking waiter counter underflow");
            }
        }
        // SAFETY: TASK_COMPLETED is published with Release only after the
        // result slot has been written. This handle is consumed by get(), so
        // the result can be taken at most once through this path.
        unsafe { self.inner.take_result() }
    }

    /// Returns whether the task has reported completion.
    ///
    /// # Returns
    ///
    /// `true` after the task runner has produced or abandoned its final result.
    #[inline]
    pub fn is_done(&self) -> bool {
        self.inner.status.load(Ordering::Acquire) == TASK_COMPLETED
    }

    /// Attempts to cancel the task.
    ///
    /// Cancellation can only win before the runner marks the task as started.
    /// It cannot interrupt a task that is already running on an OS thread.
    ///
    /// # Returns
    ///
    /// `true` if the task was cancelled before it started, or `false` if the
    /// task was already running or completed.
    #[inline]
    pub fn cancel(&self) -> bool {
        TaskCompletion {
            inner: Arc::clone(&self.inner),
        }
        .cancel()
    }
}

impl<R, E> Future for TaskHandle<R, E> {
    type Output = TaskResult<R, E>;

    /// Polls this handle for the accepted task's final result.
    ///
    /// # Parameters
    ///
    /// * `cx` - Async task context used to store the current waker while the
    ///   task is pending.
    ///
    /// # Returns
    ///
    /// `Poll::Ready` with the final task result after completion, or
    /// `Poll::Pending` while the task is still running or waiting to start.
    ///
    /// # Panics
    ///
    /// Panics if the shared state says the task completed but no final result
    /// is stored. That indicates an internal executor bug.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.inner.status.load(Ordering::Acquire) == TASK_COMPLETED {
            // SAFETY: the completed status guarantees the result has been
            // written, and Future::poll returns Ready only once after taking it.
            return Poll::Ready(unsafe { self.inner.take_result() });
        }
        let completed = self.inner.wait_state.write(|state| {
            if self.inner.status.load(Ordering::Acquire) == TASK_COMPLETED {
                true
            } else {
                state.waker = Some(cx.waker().clone());
                false
            }
        });
        if completed {
            // SAFETY: the second completed check ran while no new waker could
            // be registered concurrently through this handle state.
            Poll::Ready(unsafe { self.inner.take_result() })
        } else {
            Poll::Pending
        }
    }
}

impl<R, E> TaskHandleInner<R, E> {
    /// Notifies every waiter that the shared task state may have changed.
    ///
    /// This wakes blocking waiters parked in [`TaskHandle::get`].
    fn notify_completion(&self) {
        self.wait_state.notify_all();
    }

    /// Stores the final task result before publishing completion.
    ///
    /// # Parameters
    ///
    /// * `result` - Final result to place in the one-shot slot.
    ///
    /// # Safety
    ///
    /// Caller must own the transition to `TASK_PUBLISHING`, ensuring no other
    /// completion path can write this slot concurrently.
    unsafe fn store_result(&self, result: TaskResult<R, E>) {
        // SAFETY: guaranteed by the function contract.
        unsafe {
            *self.result.get() = Some(result);
        }
    }

    /// Takes the final task result after completion.
    ///
    /// # Returns
    ///
    /// Final task result stored by the completion path.
    ///
    /// # Safety
    ///
    /// Caller must observe `TASK_COMPLETED` with acquire ordering and must
    /// ensure the handle result is consumed at most once.
    unsafe fn take_result(&self) -> TaskResult<R, E> {
        // SAFETY: guaranteed by the function contract.
        unsafe {
            (*self.result.get())
                .take()
                .expect("task handle completed without a result")
        }
    }
}

impl<R, E> Clone for TaskCompletion<R, E> {
    /// Clones the completion endpoint for mutually exclusive finish paths.
    ///
    /// # Returns
    ///
    /// A completion endpoint sharing the same task state.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<R, E> TaskCompletion<R, E> {
    /// Marks the task as started if it was not cancelled first.
    ///
    /// # Returns
    ///
    /// `true` if the runner should execute the task, or `false` if the task was
    /// already completed through cancellation.
    pub fn start(&self) -> bool {
        self.inner
            .status
            .compare_exchange(
                TASK_PENDING,
                TASK_RUNNING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    /// Completes the task with its final result.
    ///
    /// If another path has already completed the task, this result is ignored.
    ///
    /// # Parameters
    ///
    /// * `result` - Final task result to publish if the task is not already
    ///   completed.
    pub fn complete(&self, result: TaskResult<R, E>) {
        if self.begin_publish_completion() {
            self.publish_result(result);
        }
    }

    /// Starts the task and completes it with a lazily produced result.
    ///
    /// The supplied closure is executed only if this completion endpoint wins
    /// the start race. If the handle was cancelled first, the closure is not
    /// called and the existing cancellation result is preserved.
    ///
    /// # Parameters
    ///
    /// * `task` - Closure that runs the accepted task and returns its final
    ///   result.
    ///
    /// # Returns
    ///
    /// `true` if the closure was executed and its result was published, or
    /// `false` if the task had already been completed by cancellation.
    pub fn start_and_complete<F>(&self, task: F) -> bool
    where
        F: FnOnce() -> TaskResult<R, E>,
    {
        if !self.start() {
            return false;
        }
        self.complete(task());
        true
    }

    /// Starts and completes a uniquely owned executor task.
    ///
    /// The supplied closure is executed only if this endpoint wins the start
    /// race. Unlike [`Self::start_and_complete`], this crate-internal path
    /// assumes the executor owns the only completion endpoint that can publish
    /// a non-cancellation result, so it can skip the second completion CAS.
    ///
    /// # Parameters
    ///
    /// * `task` - Closure that runs the accepted task and returns its final
    ///   result.
    ///
    /// # Returns
    ///
    /// `true` if the closure was executed and its result was published, or
    /// `false` if cancellation completed the task before it started.
    pub(crate) fn start_and_complete_unique<F>(&self, task: F) -> bool
    where
        F: FnOnce() -> TaskResult<R, E>,
    {
        if !self.start() {
            return false;
        }
        self.publish_unique_started_result(task());
        true
    }

    /// Cancels the task if it has not started yet.
    ///
    /// # Returns
    ///
    /// `true` if this call published a cancellation result, or `false` if the
    /// task was already started or completed.
    pub fn cancel(&self) -> bool {
        if self
            .inner
            .status
            .compare_exchange(
                TASK_PENDING,
                TASK_PUBLISHING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return false;
        }
        self.publish_result(Err(TaskExecutionError::Cancelled));
        true
    }

    /// Claims the right to publish a non-cancellation completion result.
    ///
    /// # Returns
    ///
    /// `true` if this completion endpoint may store the result.
    fn begin_publish_completion(&self) -> bool {
        let mut current = self.inner.status.load(Ordering::Acquire);
        loop {
            match current {
                TASK_PENDING | TASK_RUNNING => {
                    match self.inner.status.compare_exchange_weak(
                        current,
                        TASK_PUBLISHING,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    ) {
                        Ok(_) => return true,
                        Err(actual) => current = actual,
                    }
                }
                TASK_PUBLISHING | TASK_COMPLETED => return false,
                _ => unreachable!("invalid task handle status"),
            }
        }
    }

    /// Publishes the result for a uniquely started executor task.
    ///
    /// # Parameters
    ///
    /// * `result` - Terminal result to store.
    fn publish_unique_started_result(&self, result: TaskResult<R, E>) {
        debug_assert_eq!(
            self.inner.status.load(Ordering::Acquire),
            TASK_RUNNING,
            "unique task completion must start from running state"
        );
        self.publish_result(result);
    }

    /// Stores the final result and wakes registered waiters.
    ///
    /// # Parameters
    ///
    /// * `result` - Terminal result to store.
    fn publish_result(&self, result: TaskResult<R, E>) {
        // SAFETY: begin_publish_completion(), cancel(), or the crate-internal
        // unique completion path has given this endpoint exclusive ownership
        // of the result slot.
        unsafe {
            self.inner.store_result(result);
        }
        self.inner.status.store(TASK_COMPLETED, Ordering::Release);
        let (notify_blocking_waiters, waker) = self
            .inner
            .wait_state
            .write(|state| (state.blocking_waiters > 0, state.waker.take()));
        if notify_blocking_waiters {
            self.inner.notify_completion();
        }
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}
