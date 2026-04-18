/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use super::executor::Executor;

/// Executes synchronous tasks immediately on the caller thread.
///
/// This executor is useful for deterministic tests and simple scenarios where
/// task submission and execution should happen in the same call stack.
#[derive(Debug, Default, Clone, Copy)]
pub struct DirectExecutor;

impl Executor for DirectExecutor {
    /// Executes the task inline on the current thread.
    #[inline]
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        task();
    }
}
