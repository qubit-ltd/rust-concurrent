/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
};

use super::{
    executor::Executor,
    runnable::BoxRunnable,
};

/// Shutdownable executor trait
///
/// Provides lifecycle management functionality similar to the JDK `ExecutorService` interface.
/// Executors implementing this trait can be gracefully shut down, waiting for executing tasks to complete.
///
/// # Features
///
/// - Graceful shutdown: The `shutdown()` method stops accepting new tasks but waits for submitted tasks to complete
/// - Immediate shutdown: The `shutdown_now()` method attempts to stop all executing tasks
/// - State queries: Can query whether the executor is shut down or terminated
///
/// # Examples
///
/// ```rust
/// use qubit_concurrent::{Executor, ExecutorService};
///
/// #[tokio::main]
/// async fn main() {
///     let executor = create_executor();
///
///     // Submit task
///     executor.execute(Box::new(|| {
///         println!("Task executing");
///     }));
///
///     // Graceful shutdown
///     executor.shutdown();
///
///     // Wait for all tasks to complete
///     executor.await_termination().await;
///
///     assert!(executor.is_terminated());
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub trait ExecutorService: Executor {
    /// Initiates an orderly shutdown
    ///
    /// Initiates an orderly shutdown in which previously submitted tasks are executed,
    /// but no new tasks will be accepted. Invocation has no additional effect if already shut down.
    ///
    /// This method does not wait for previously submitted tasks to complete execution.
    /// Use `await_termination()` to do that.
    fn shutdown(&self);

    /// Attempts to stop all actively executing tasks
    ///
    /// Attempts to stop all actively executing tasks, halts the processing of waiting tasks,
    /// and returns a list of the tasks that were awaiting execution.
    ///
    /// This method does not wait for actively executing tasks to terminate.
    /// Use `await_termination()` to do that.
    ///
    /// # Returns
    ///
    /// List of tasks that never commenced execution.
    ///
    /// Returned tasks are fallible `qubit-function` runnables. Current executor
    /// service implementations in this crate execute work immediately, so they
    /// return an empty list.
    fn shutdown_now(&self) -> Vec<BoxRunnable<Infallible>>;

    /// Returns true if this executor has been shut down
    ///
    /// # Returns
    ///
    /// `true` if this executor has been shut down
    fn is_shutdown(&self) -> bool;

    /// Returns true if all tasks have completed following shut down
    ///
    /// # Returns
    ///
    /// `true` if all tasks have completed following shut down.
    /// Note that `is_terminated()` is never `true` unless either
    /// `shutdown()` or `shutdown_now()` was called first.
    fn is_terminated(&self) -> bool;

    /// Blocks until all tasks have completed execution
    ///
    /// Blocks until all tasks have completed execution after a shutdown request,
    /// or the timeout occurs, or the current task is interrupted, whichever happens first.
    ///
    /// # Returns
    ///
    /// A future that completes when all tasks have finished
    fn await_termination(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
