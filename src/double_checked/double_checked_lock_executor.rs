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

/// Initial builder for [`DoubleCheckedLockExecutor`].
///
/// This state has no lock yet. Call [`Self::on`] to attach the lock.
///
/// # Author
///
/// Haixing Hu
#[derive(Debug, Default, Clone)]
pub struct DoubleCheckedLockExecutorBuilder {
    /// Optional logger carried forward to later builder states.
    logger: Option<ExecutionLogger>,
}

/// Builder state after a lock has been attached.
///
/// Call [`Self::when`] to configure the required condition tester.
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
pub struct DoubleCheckedLockExecutorLockBuilder<L, T> {
    /// The lock to store in the executor.
    lock: L,

    /// Optional logger carried forward to the ready builder state.
    logger: Option<ExecutionLogger>,

    /// Carries the protected data type.
    _phantom: PhantomData<fn() -> T>,
}

/// Builder state after the required condition tester has been configured.
///
/// This state can configure prepare lifecycle callbacks and build the final
/// [`DoubleCheckedLockExecutor`].
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
pub struct DoubleCheckedLockExecutorReadyBuilder<L, T> {
    /// The lock to store in the executor.
    lock: L,

    /// Required condition tester.
    tester: ArcTester,

    /// Optional logger used by the executor.
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

impl DoubleCheckedLockExecutor<(), ()> {
    /// Creates a builder for a reusable double-checked lock executor.
    ///
    /// # Returns
    ///
    /// A builder in the initial state. Attach a lock with
    /// [`DoubleCheckedLockExecutorBuilder::on`], then configure a tester with
    /// [`DoubleCheckedLockExecutorLockBuilder::when`].
    #[inline]
    pub fn builder() -> DoubleCheckedLockExecutorBuilder {
        DoubleCheckedLockExecutorBuilder::default()
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
    fn log_unmet_condition(&self) {
        if let Some(ref logger) = self.logger {
            logger.log_unmet_message();
        }
    }
}

impl DoubleCheckedLockExecutorBuilder {
    /// Configures logging before the lock is attached.
    ///
    /// # Parameters
    ///
    /// * `level` - The log level used for double-checked execution events.
    /// * `message` - The message logged when the condition is not met.
    ///
    /// # Returns
    ///
    /// This builder with logging configured.
    #[inline]
    pub fn logger(mut self, level: log::Level, message: &str) -> Self {
        self.logger = Some(ExecutionLogger::new(level, message));
        self
    }

    /// Attaches the lock protected by this executor.
    ///
    /// # Parameters
    ///
    /// * `lock` - The lock handle. Arc-based lock wrappers can be cloned and
    ///   stored here for reusable execution.
    ///
    /// # Returns
    ///
    /// The builder state that can configure the required tester.
    #[inline]
    pub fn on<L, T>(self, lock: L) -> DoubleCheckedLockExecutorLockBuilder<L, T>
    where
        L: Lock<T>,
    {
        DoubleCheckedLockExecutorLockBuilder {
            lock,
            logger: self.logger,
            _phantom: PhantomData,
        }
    }
}

impl<L, T> DoubleCheckedLockExecutorLockBuilder<L, T>
where
    L: Lock<T>,
{
    /// Configures logging after the lock is attached.
    ///
    /// # Parameters
    ///
    /// * `level` - The log level used for double-checked execution events.
    /// * `message` - The message logged when the condition is not met.
    ///
    /// # Returns
    ///
    /// This builder with logging configured.
    #[inline]
    pub fn logger(mut self, level: log::Level, message: &str) -> Self {
        self.logger = Some(ExecutionLogger::new(level, message));
        self
    }

    /// Configures the required double-checked condition.
    ///
    /// The tester is executed outside and inside the lock. State read by the
    /// outside check must be safe to access without this executor's lock.
    ///
    /// # Parameters
    ///
    /// * `tester` - The reusable condition tester.
    ///
    /// # Returns
    ///
    /// The builder state that can configure prepare callbacks and build the
    /// executor.
    #[inline]
    pub fn when<Tst>(self, tester: Tst) -> DoubleCheckedLockExecutorReadyBuilder<L, T>
    where
        Tst: Tester + Send + Sync + 'static,
    {
        DoubleCheckedLockExecutorReadyBuilder {
            lock: self.lock,
            tester: tester.into_arc(),
            logger: self.logger,
            prepare_action: None,
            rollback_prepare_action: None,
            commit_prepare_action: None,
            _phantom: PhantomData,
        }
    }
}

impl<L, T> DoubleCheckedLockExecutorReadyBuilder<L, T>
where
    L: Lock<T>,
{
    /// Configures logging after the tester is set.
    ///
    /// # Parameters
    ///
    /// * `level` - The log level used for double-checked execution events.
    /// * `message` - The message logged when the condition is not met.
    ///
    /// # Returns
    ///
    /// This builder with logging configured.
    #[inline]
    pub fn logger(mut self, level: log::Level, message: &str) -> Self {
        self.logger = Some(ExecutionLogger::new(level, message));
        self
    }

    /// Sets the prepare action.
    ///
    /// The action runs after the first condition check succeeds and before the
    /// lock is acquired. If it succeeds, the executor will later run either
    /// rollback or commit according to the final task result.
    ///
    /// # Parameters
    ///
    /// * `prepare_action` - The fallible action to run before locking.
    ///
    /// # Returns
    ///
    /// This builder with prepare configured.
    #[inline]
    pub fn prepare<Rn, E>(mut self, prepare_action: Rn) -> Self
    where
        Rn: Runnable<E> + Send + 'static,
        E: Display + Send + 'static,
    {
        let mut action = prepare_action;
        self.prepare_action = Some(ArcRunnable::new(move || {
            action.run().map_err(|error| error.to_string())
        }));
        self
    }

    /// Sets the rollback action for a successfully completed prepare action.
    ///
    /// # Parameters
    ///
    /// * `rollback_prepare_action` - The action to run if the second condition
    ///   check or task execution fails after prepare succeeds.
    ///
    /// # Returns
    ///
    /// This builder with prepare rollback configured.
    #[inline]
    pub fn rollback_prepare<Rn, E>(mut self, rollback_prepare_action: Rn) -> Self
    where
        Rn: Runnable<E> + Send + 'static,
        E: Display + Send + 'static,
    {
        let mut action = rollback_prepare_action;
        self.rollback_prepare_action = Some(ArcRunnable::new(move || {
            action.run().map_err(|error| error.to_string())
        }));
        self
    }

    /// Sets the commit action for a successfully completed prepare action.
    ///
    /// # Parameters
    ///
    /// * `commit_prepare_action` - The action to run if the task succeeds after
    ///   prepare succeeds.
    ///
    /// # Returns
    ///
    /// This builder with prepare commit configured.
    #[inline]
    pub fn commit_prepare<Rn, E>(mut self, commit_prepare_action: Rn) -> Self
    where
        Rn: Runnable<E> + Send + 'static,
        E: Display + Send + 'static,
    {
        let mut action = commit_prepare_action;
        self.commit_prepare_action = Some(ArcRunnable::new(move || {
            action.run().map_err(|error| error.to_string())
        }));
        self
    }

    /// Builds the reusable executor.
    ///
    /// # Returns
    ///
    /// A [`DoubleCheckedLockExecutor`] containing the configured lock, tester,
    /// logger, and prepare lifecycle callbacks.
    #[inline]
    pub fn build(self) -> DoubleCheckedLockExecutor<L, T> {
        DoubleCheckedLockExecutor {
            lock: self.lock,
            tester: self.tester,
            logger: self.logger,
            prepare_action: self.prepare_action,
            rollback_prepare_action: self.rollback_prepare_action,
            commit_prepare_action: self.commit_prepare_action,
            _phantom: PhantomData,
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
