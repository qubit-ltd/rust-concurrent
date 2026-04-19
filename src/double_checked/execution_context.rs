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
use std::error::Error;

use qubit_function::{
    BoxSupplierOnce,
    SupplierOnce,
};

use crate::double_checked::{
    execution_result::ExecutionResult,
    ExecutionLogger,
};

/// Execution context (state after task execution)
///
/// This type provides rollback and result retrieval functionality after
/// task execution. It holds the execution status and optionally sets
/// rollback operations.
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
    rollback_action: Option<BoxSupplierOnce<Result<(), Box<dyn Error + Send + Sync>>>>,
    rollback_on_unmet: bool,
    logger: Option<ExecutionLogger>,
}

impl<T, E> ExecutionContext<T, E>
where
    E: std::fmt::Display,
{
    /// Creates a new execution context with condition-unmet rollback policy.
    ///
    /// # Arguments
    ///
    /// * `result` - The execution result
    /// * `rollback_on_unmet` - Whether rollback should run when result is
    ///   `ConditionNotMet`
    /// * `logger` - Optional execution logger (used for rollback failure lines)
    #[inline]
    pub(super) fn new(
        result: ExecutionResult<T, E>,
        rollback_on_unmet: bool,
        logger: Option<ExecutionLogger>,
    ) -> Self {
        Self {
            result,
            rollback_action: None,
            rollback_on_unmet,
            logger,
        }
    }

    /// Sets rollback action (optional, only executed on failure)
    ///
    /// # Arguments
    ///
    /// * `rollback_action` - Any type that implements
    ///   `SupplierOnce<Result<(), RE>>`
    ///
    /// # Note
    ///
    /// Rollback is only set and executed when `result` is `Failed`.
    pub fn rollback<S, RE>(mut self, rollback_action: S) -> Self
    where
        S: SupplierOnce<Result<(), RE>> + 'static,
        RE: Error + Send + Sync + 'static,
    {
        let should_register = matches!(self.result, ExecutionResult::Failed(_))
            || (self.rollback_on_unmet && matches!(self.result, ExecutionResult::ConditionNotMet));
        if should_register {
            let boxed = rollback_action.into_box();
            self.rollback_action = Some(BoxSupplierOnce::new(move || {
                boxed
                    .get()
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
            }));
        }
        self
    }

    /// Gets the execution result (consumes the context)
    ///
    /// If rollback is set and execution failed, rollback will be executed
    /// before returning the result.
    ///
    /// If rollback execution fails, the error in the returned result will be
    /// updated to `RollbackFailed`.
    pub fn get_result(mut self) -> ExecutionResult<T, E> {
        let should_execute = matches!(self.result, ExecutionResult::Failed(_))
            || (self.rollback_on_unmet && matches!(self.result, ExecutionResult::ConditionNotMet));
        if should_execute {
            let original = match &self.result {
                ExecutionResult::Failed(error) => Some(error.to_string()),
                ExecutionResult::ConditionNotMet => Some("Condition not met".to_string()),
                ExecutionResult::Success(_) => None,
            };
            if let (Some(rollback_action), Some(original)) = (self.rollback_action.take(), original) {
                if let Err(rollback_error) = rollback_action.get() {
                    if let Some(ref log) = self.logger {
                        log.log_rollback_failed(&rollback_error);
                    } else {
                        log::error!("Rollback action failed: {}", rollback_error);
                    }
                    self.result = ExecutionResult::rollback_failed(
                        original,
                        rollback_error.to_string(),
                    );
                }
            }
        }
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
