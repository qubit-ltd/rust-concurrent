/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Builder state after a lock has been attached for
//! [`super::DoubleCheckedLockExecutor`].
//!
//! # Author
//!
//! Haixing Hu

use std::marker::PhantomData;

use qubit_function::Tester;

use super::{
    executor_ready_builder::ExecutorReadyBuilder,
    ExecutionLogger,
};
use crate::lock::Lock;

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
pub struct ExecutorLockBuilder<L, T> {
    /// The lock to store in the executor.
    pub(in crate::double_checked) lock: L,

    /// Optional logger carried forward to the ready builder state.
    pub(in crate::double_checked) logger: Option<ExecutionLogger>,

    /// Carries the protected data type.
    pub(in crate::double_checked) _phantom: PhantomData<fn() -> T>,
}

impl<L, T> ExecutorLockBuilder<L, T>
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
    pub fn when<Tst>(self, tester: Tst) -> ExecutorReadyBuilder<L, T>
    where
        Tst: Tester + Send + Sync + 'static,
    {
        ExecutorReadyBuilder {
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
