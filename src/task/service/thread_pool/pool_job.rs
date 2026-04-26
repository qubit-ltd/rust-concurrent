/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use qubit_function::Callable;

use crate::task::{TaskCompletion, task_runner::run_callable};

/// Type-erased pool job with a cancellation path for queued work.
///
/// `PoolJob` is a low-level extension point for building custom services on
/// top of [`super::thread_pool::ThreadPool`]. The pool calls the run callback after a worker takes
/// the job, or the cancel callback if the job is still queued during immediate
/// shutdown.
pub struct PoolJob {
    /// Type-erased job body that can be consumed by either run or cancel.
    body: Box<dyn PoolJobBody>,
}

/// Consuming callbacks supported by a queued pool job.
trait PoolJobBody: Send + 'static {
    /// Runs this job after a worker claims it.
    fn run(self: Box<Self>);

    /// Cancels this job before a worker starts it.
    fn cancel(self: Box<Self>);
}

/// Pool job backed by separate run and cancel closures.
struct ClosurePoolJob {
    /// Callback executed once a worker starts the job.
    run: Box<dyn FnOnce() + Send + 'static>,
    /// Callback executed if the job is cancelled before a worker starts it.
    cancel: Box<dyn FnOnce() + Send + 'static>,
}

impl PoolJobBody for ClosurePoolJob {
    /// Runs the stored run callback.
    fn run(self: Box<Self>) {
        (self.run)();
    }

    /// Runs the stored cancel callback.
    fn cancel(self: Box<Self>) {
        (self.cancel)();
    }
}

/// Pool job backed directly by a task completion endpoint.
struct TaskPoolJob<C, R, E> {
    /// Callable to execute when the job is claimed.
    task: C,
    /// Completion endpoint updated by run or cancel.
    completion: TaskCompletion<R, E>,
}

impl<C, R, E> PoolJobBody for TaskPoolJob<C, R, E>
where
    C: Callable<R, E> + Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
{
    /// Runs the callable and publishes its result.
    fn run(self: Box<Self>) {
        let Self { task, completion } = *self;
        completion.start_and_complete_unique(|| run_callable(task));
    }

    /// Publishes cancellation if the task has not started.
    fn cancel(self: Box<Self>) {
        self.completion.cancel();
    }
}

impl PoolJob {
    /// Creates a pool job from run and cancel callbacks.
    ///
    /// # Parameters
    ///
    /// * `run` - Callback executed once a worker starts this job.
    /// * `cancel` - Callback executed if this job is cancelled while queued.
    ///
    /// # Returns
    ///
    /// A type-erased job accepted by [`super::thread_pool::ThreadPool::submit_job`].
    pub fn new(
        run: Box<dyn FnOnce() + Send + 'static>,
        cancel: Box<dyn FnOnce() + Send + 'static>,
    ) -> Self {
        Self {
            body: Box::new(ClosurePoolJob { run, cancel }),
        }
    }

    /// Creates a pool job directly from a callable and completion endpoint.
    ///
    /// # Parameters
    ///
    /// * `task` - Callable executed once the job is claimed.
    /// * `completion` - Completion endpoint used for success, failure, panic,
    ///   or queued cancellation.
    ///
    /// # Returns
    ///
    /// A type-erased job that avoids allocating separate run and cancel
    /// closures.
    pub(crate) fn from_task<C, R, E>(task: C, completion: TaskCompletion<R, E>) -> Self
    where
        C: Callable<R, E> + Send + 'static,
        R: Send + 'static,
        E: Send + 'static,
    {
        Self {
            body: Box::new(TaskPoolJob { task, completion }),
        }
    }

    /// Runs this job if it has not been cancelled first.
    ///
    /// Consumes the job and invokes the run callback at most once.
    pub(crate) fn run(self) {
        self.body.run();
    }

    /// Cancels this queued job if it has not been run first.
    ///
    /// Consumes the job and invokes the cancellation callback at most once.
    pub(crate) fn cancel(self) {
        self.body.cancel();
    }
}
