/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Initial builder for [`super::DoubleCheckedLockExecutor`].
//!
//! # Author
//!
//! Haixing Hu

use std::marker::PhantomData;

use super::{
    executor_lock_builder::ExecutorLockBuilder,
    ExecutionLogger,
};
use crate::lock::Lock;

/// Initial builder for [`super::DoubleCheckedLockExecutor`].
///
/// This state has no lock yet. Call [`Self::on`] to attach the lock.
///
/// # Author
///
/// Haixing Hu
#[derive(Debug, Default, Clone)]
pub struct ExecutorBuilder {
    /// Optional logger carried forward to later builder states.
    logger: Option<ExecutionLogger>,
}

impl ExecutorBuilder {
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
    pub fn on<L, T>(self, lock: L) -> ExecutorLockBuilder<L, T>
    where
        L: Lock<T>,
    {
        ExecutorLockBuilder {
            lock,
            logger: self.logger,
            _phantom: PhantomData,
        }
    }
}
