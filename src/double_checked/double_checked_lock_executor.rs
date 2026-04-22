/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Double-Checked Lock Executor
//!
//! Provides a reusable executor for double-checked locking workflows.
//!
//! # Author
//!
//! Haixing Hu

use std::{
    fmt::Display,
    marker::PhantomData,
};

use qubit_function::{
    ArcRunnable,
    ArcTester,
    Callable,
    CallableWith,
    Runnable,
    RunnableWith,
    Tester,
};

use super::{
    ExecutionContext,
    ExecutionLogger,
    ExecutionResult,
};
use crate::{
    lock::Lock,
    task::executor::Executor,
};

/// Reusable double-checked lock executor.
///
/// The executor owns the lock handle, condition tester, and optional prepare
/// lifecycle callbacks. Each execution performs:
///
/// 1. A first condition check outside the lock.
/// 2. Optional prepare action.
/// 3. Lock acquisition.
/// 4. A second condition check inside the lock.
/// 5. The submitted task.
/// 6. Optional prepare commit or rollback after the lock is released.
///
/// The tester is intentionally run both outside and inside the lock. Any state
/// read by the first check must therefore use atomics or another synchronization
/// mechanism that is safe without this executor's lock.
///
/// # Type Parameters
///
/// * `L` - The lock type implementing [`Lock<T>`].
/// * `T` - The data type protected by the lock.
///
/// # Author
///
/// Haixing Hu
#[derive(Clone)]
pub struct DoubleCheckedLockExecutor<L = (), T = ()> {
    /// The lock protecting the target data.
    lock: L,

    /// Condition checked before and after acquiring the lock.
    tester: ArcTester,

    /// Optional logger used for unmet conditions and prepare failures.
    logger: Option<ExecutionLogger>,

    /// Optional action executed after the first check and before locking.
    prepare_action: Option<ArcRunnable<String>>,

    /// Optional action executed when prepare must be rolled back.
    rollback_prepare_action: Option<ArcRunnable<String>>,

    /// Optional action executed when prepare should be committed.
    commit_prepare_action: Option<ArcRunnable<String>>,

    /// Carries the protected data type.
    _phantom: PhantomData<fn() -> T>,
}

impl<L, T> DoubleCheckedLockExecutor<L, T> {
    /// Assembles an executor from builder state (lock, tester, optional hooks).
    ///
    /// # Parameters
    ///
    /// * `lock` - Lock protecting the target data.
    /// * `tester` - Condition evaluated before and after lock acquisition.
    /// * `logger` - Optional logger used for unmet conditions and prepare
    ///   lifecycle failures.
    /// * `prepare_action` - Optional action run before lock acquisition.
    /// * `rollback_prepare_action` - Optional action run when a completed
    ///   prepare action must be rolled back.
    /// * `commit_prepare_action` - Optional action run when a completed
    ///   prepare action should be committed.
    ///
    /// # Returns
    ///
    /// A reusable executor containing the supplied builder state.
    #[inline]
    pub(in crate::double_checked) fn from_builder_state(
        lock: L,
        tester: ArcTester,
        logger: Option<ExecutionLogger>,
        prepare_action: Option<ArcRunnable<String>>,
        rollback_prepare_action: Option<ArcRunnable<String>>,
        commit_prepare_action: Option<ArcRunnable<String>>,
    ) -> Self {
        Self {
            lock,
            tester,
            logger,
            prepare_action,
            rollback_prepare_action,
            commit_prepare_action,
            _phantom: PhantomData,
        }
    }
}

impl<L, T> DoubleCheckedLockExecutor<L, T>
where
    L: Lock<T>,
{
    /// Executes a zero-argument callable while holding the write lock.
    ///
    /// This method is the [`Executor`] style API. Use [`Self::call_with`] when
    /// the task needs direct mutable access to the protected data.
    ///
    /// # Parameters
    ///
    /// * `task` - The callable task to execute after both condition checks pass.
    ///
    /// # Returns
    ///
    /// An [`ExecutionContext`] containing success, unmet-condition, or failure
    /// information.
    #[inline]
    pub fn call<C, R, E>(&self, task: C) -> ExecutionContext<R, E>
    where
        C: Callable<R, E> + Send + 'static,
        R: Send + 'static,
        E: Display + Send + 'static,
    {
        self.call_callable(task)
    }

    /// Executes a zero-argument runnable while holding the write lock.
    ///
    /// # Parameters
    ///
    /// * `task` - The runnable task to execute after both condition checks pass.
    ///
    /// # Returns
    ///
    /// An [`ExecutionContext`] containing success, unmet-condition, or failure
    /// information.
    #[inline]
    pub fn execute<Rn, E>(&self, task: Rn) -> ExecutionContext<(), E>
    where
        Rn: Runnable<E> + Send + 'static,
        E: Display + Send + 'static,
    {
        let mut task = task;
        let result = self.execute_with_write_lock(move |_data| task.run());
        ExecutionContext::new(result)
    }

    /// Executes a callable with mutable access to the protected data.
    ///
    /// # Parameters
    ///
    /// * `task` - The callable receiving `&mut T` after both condition checks
    ///   pass.
    ///
    /// # Returns
    ///
    /// An [`ExecutionContext`] containing success, unmet-condition, or failure
    /// information.
    #[inline]
    pub fn call_with<C, R, E>(&self, task: C) -> ExecutionContext<R, E>
    where
        C: CallableWith<T, R, E> + Send + 'static,
        R: Send + 'static,
        E: Display + Send + 'static,
    {
        let mut task = task;
        let result = self.execute_with_write_lock(move |data| task.call_with(data));
        ExecutionContext::new(result)
    }

    /// Executes a runnable with mutable access to the protected data.
    ///
    /// # Parameters
    ///
    /// * `task` - The runnable receiving `&mut T` after both condition checks
    ///   pass.
    ///
    /// # Returns
    ///
    /// An [`ExecutionContext`] containing success, unmet-condition, or failure
    /// information.
    #[inline]
    pub fn execute_with<Rn, E>(&self, task: Rn) -> ExecutionContext<(), E>
    where
        Rn: RunnableWith<T, E> + Send + 'static,
        E: Display + Send + 'static,
    {
        let mut task = task;
        let result = self.execute_with_write_lock(move |data| task.run_with(data));
        ExecutionContext::new(result)
    }

    /// Executes a zero-argument callable through the double-checked sequence.
    ///
    /// # Parameters
    ///
    /// * `task` - Callable to run when both condition checks pass.
    ///
    /// # Returns
    ///
    /// An [`ExecutionContext`] containing success, unmet-condition, or failure
    /// information.
    fn call_callable<C, R, E>(&self, task: C) -> ExecutionContext<R, E>
    where
        C: Callable<R, E> + Send + 'static,
        R: Send + 'static,
        E: Display + Send + 'static,
    {
        let mut task = task;
        let result = self.execute_with_write_lock(move |_data| task.call());
        ExecutionContext::new(result)
    }

    /// Runs the configured double-checked sequence under a write lock.
    ///
    /// # Parameters
    ///
    /// * `task` - The task to run with mutable access after both condition
    ///   checks pass.
    ///
    /// # Returns
    ///
    /// The final execution result, including prepare finalization.
    ///
    /// # Errors
    ///
    /// Task errors are captured as [`ExecutionResult::Failed`] with
    /// [`super::ExecutorError::TaskFailed`]. Prepare, commit, and rollback
    /// failures are also captured in the returned [`ExecutionResult`] rather
    /// than returned as a separate `Result`.
    fn execute_with_write_lock<R, E, F>(&self, task: F) -> ExecutionResult<R, E>
    where
        E: Display + Send + 'static,
        F: FnOnce(&mut T) -> Result<R, E>,
    {
        if !self.tester.test() {
            self.log_unmet_condition();
            return ExecutionResult::unmet();
        }

        let prepare_completed = match self.run_prepare_action() {
            Ok(completed) => completed,
            Err(error) => return ExecutionResult::prepare_failed(error),
        };

        let result = self.lock.write(|data| {
            if !self.tester.test() {
                self.log_unmet_condition();
                return ExecutionResult::unmet();
            }
            match task(data) {
                Ok(value) => ExecutionResult::success(value),
                Err(error) => ExecutionResult::task_failed(error),
            }
        });

        if prepare_completed {
            self.finalize_prepare(result)
        } else {
            result
        }
    }

    /// Executes the optional prepare action.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if prepare exists and succeeds, `Ok(false)` if no prepare
    /// action is configured, or `Err(message)` if prepare fails.
    ///
    /// # Errors
    ///
    /// Returns `Err(message)` when the configured prepare action returns an
    /// error. The message is already converted to [`String`].
    fn run_prepare_action(&self) -> Result<bool, String> {
        let Some(mut prepare_action) = self.prepare_action.clone() else {
            return Ok(false);
        };
        if let Err(error) = prepare_action.run() {
            if let Some(ref logger) = self.logger {
                logger.log_prepare_failed(&error);
            } else {
                log::error!("Prepare action failed: {}", error);
            }
            return Err(error);
        }
        Ok(true)
    }

    /// Commits or rolls back a successfully completed prepare action.
    ///
    /// This method runs after the write lock has been released.
    ///
    /// # Parameters
    ///
    /// * `result` - Result produced by the condition check and task execution.
    ///
    /// # Returns
    ///
    /// `result` unchanged when no finalization action fails. Returns a failed
    /// result when prepare commit or prepare rollback fails.
    fn finalize_prepare<R, E>(&self, mut result: ExecutionResult<R, E>) -> ExecutionResult<R, E>
    where
        E: Display + Send + 'static,
    {
        if result.is_success() {
            if let Some(mut commit_prepare_action) = self.commit_prepare_action.clone()
                && let Err(error) = commit_prepare_action.run()
            {
                if let Some(ref logger) = self.logger {
                    logger.log_prepare_commit_failed(&error);
                } else {
                    log::error!("Prepare commit action failed: {}", error);
                }
                result = ExecutionResult::prepare_commit_failed(error);
            }
            return result;
        }

        let original = if let ExecutionResult::Failed(error) = &result {
            error.to_string()
        } else {
            "Condition not met".to_string()
        };

        if let Some(mut rollback_prepare_action) = self.rollback_prepare_action.clone()
            && let Err(error) = rollback_prepare_action.run()
        {
            if let Some(ref logger) = self.logger {
                logger.log_prepare_rollback_failed(&error);
            } else {
                log::error!("Prepare rollback action failed: {}", error);
            }
            result = ExecutionResult::prepare_rollback_failed(original, error);
        }
        result
    }

    /// Logs that the double-checked condition was not met.
    ///
    /// This method writes through the configured logger, if any.
    fn log_unmet_condition(&self) {
        if let Some(ref logger) = self.logger {
            logger.log_unmet_message();
        }
    }
}

impl<L, T> Executor for DoubleCheckedLockExecutor<L, T>
where
    L: Lock<T> + Send + Sync,
{
    type Execution<R, E>
        = ExecutionContext<R, E>
    where
        R: Send + 'static,
        E: Display + Send + 'static;

    /// Executes the callable through the configured double-checked lock.
    ///
    /// # Parameters
    ///
    /// * `task` - Callable to run when both condition checks pass.
    ///
    /// # Returns
    ///
    /// An [`ExecutionContext`] containing success, unmet-condition, or failure
    /// information.
    #[inline]
    fn call<C, R, E>(&self, task: C) -> Self::Execution<R, E>
    where
        C: Callable<R, E> + Send + 'static,
        R: Send + 'static,
        E: Display + Send + 'static,
    {
        self.call_callable(task)
    }
}
