/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Execution Context
//!
//! Provides execution context after double-checked lock task execution.
//!
//! # Author
//!
//! Haixing Hu
use crate::double_checked::execution_result::ExecutionResult;

/// Execution context (state after task execution)
///
/// This type provides result retrieval functionality after task execution.
///
/// Prepare lifecycle callbacks are configured on
/// [`super::DoubleCheckedLockExecutor`] and are already applied before an
/// `ExecutionContext` is returned. Task closures are responsible for their own
/// rollback, cleanup, and commit logic.
///
/// # Type Parameters
///
/// * `T` - The type of the task return value
/// * `E` - The type of the task error
///
/// # Author
///
/// Haixing Hu
pub struct ExecutionContext<T, E>
where
    E: std::fmt::Display,
{
    result: ExecutionResult<T, E>,
}

impl<T, E> ExecutionContext<T, E>
where
    E: std::fmt::Display,
{
    /// Creates a new execution context.
    ///
    /// # Arguments
    ///
    /// * `result` - The execution result
    #[inline]
    pub(super) fn new(result: ExecutionResult<T, E>) -> Self {
        Self { result }
    }

    /// Gets the execution result (consumes the context)
    ///
    /// Prepare commit or rollback callbacks have already been executed by the
    /// builder before this context was created. This method does not trigger
    /// additional side effects.
    #[inline]
    pub fn get_result(self) -> ExecutionResult<T, E> {
        self.result
    }

    /// Checks the execution result (does not consume the context)
    #[inline]
    pub fn peek_result(&self) -> &ExecutionResult<T, E> {
        &self.result
    }

    /// Checks if execution was successful
    #[inline]
    pub fn is_success(&self) -> bool {
        self.result.is_success()
    }
}

// Convenience methods for cases without return values
impl<E> ExecutionContext<(), E>
where
    E: std::fmt::Display,
{
    /// Completes execution (for operations without return values)
    ///
    /// Returns whether the execution was successful
    #[inline]
    pub fn finish(self) -> bool {
        let result = self.get_result();
        result.is_success()
    }
}
