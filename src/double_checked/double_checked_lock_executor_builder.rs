/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Builder types for [`super::DoubleCheckedLockExecutor`].
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
    Runnable,
    Tester,
};

use super::ExecutionLogger;
use crate::lock::Lock;

use super::double_checked_lock_executor::DoubleCheckedLockExecutor;

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
    /// Errors returned by this action are converted to [`String`] and reported
    /// by execution methods as [`super::ExecutionResult::Failed`].
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
    /// Errors returned by this action are converted to [`String`] and replace
    /// the original execution result with a prepare-rollback failure.
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
    /// Errors returned by this action are converted to [`String`] and replace
    /// an otherwise successful execution result with a prepare-commit failure.
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
        DoubleCheckedLockExecutor::from_builder_state(
            self.lock,
            self.tester,
            self.logger,
            self.prepare_action,
            self.rollback_prepare_action,
            self.commit_prepare_action,
        )
    }
}
