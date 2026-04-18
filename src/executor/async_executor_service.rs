/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::{
    future::Future,
    pin::Pin,
};

use super::async_executor::AsyncExecutor;

/// Shutdownable asynchronous executor trait
///
/// Provides lifecycle management functionality for asynchronous executors.
/// This is the async counterpart of `ExecutorService`, specifically for managing
/// the lifecycle of asynchronous task executors.
///
/// # Features
///
/// - Graceful shutdown: The `shutdown()` method stops accepting new async tasks but waits for submitted tasks to complete
/// - Immediate shutdown: The `shutdown_now()` method attempts to stop all executing async tasks
/// - State queries: Can query whether the async executor is shut down or terminated
///
/// # Use Cases
///
/// - Async I/O service shutdown: Gracefully shut down async network or file I/O services
/// - Application cleanup: Ensure all async tasks complete before application exit
/// - Resource management: Control the lifecycle of async task pools
///
/// # Examples
///
/// ```rust
/// use qubit_concurrent::{AsyncExecutor, AsyncExecutorService};
///
/// #[tokio::main]
/// async fn main() {
///     let executor = create_async_executor();
///
///     // Submit async task
///     executor.spawn(async {
///         println!("Async task executing");
///     });
///
///     // Graceful shutdown
///     executor.shutdown();
///
///     // Wait for all async tasks to complete
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
pub trait AsyncExecutorService: AsyncExecutor {
    /// Initiates an orderly shutdown
    ///
    /// Initiates an orderly shutdown in which previously submitted async tasks are executed,
    /// but no new tasks will be accepted. Invocation has no additional effect if already shut down.
    ///
    /// This method does not wait for previously submitted tasks to complete execution.
    /// Use `await_termination()` to do that.
    fn shutdown(&self);

    /// Attempts to stop all actively executing async tasks
    ///
    /// Attempts to stop all actively executing async tasks and halts the processing of waiting tasks.
    /// Unlike the synchronous version, this method does not return a list of pending tasks
    /// because async tasks cannot be easily captured once spawned.
    ///
    /// This method does not wait for actively executing tasks to terminate.
    /// Use `await_termination()` to do that.
    fn shutdown_now(&self);

    /// Returns true if this async executor has been shut down
    ///
    /// # Returns
    ///
    /// `true` if this async executor has been shut down
    fn is_shutdown(&self) -> bool;

    /// Returns true if all async tasks have completed following shut down
    ///
    /// # Returns
    ///
    /// `true` if all async tasks have completed following shut down.
    /// Note that `is_terminated()` is never `true` unless either
    /// `shutdown()` or `shutdown_now()` was called first.
    fn is_terminated(&self) -> bool;

    /// Blocks until all async tasks have completed execution
    ///
    /// Blocks until all async tasks have completed execution after a shutdown request,
    /// or the timeout occurs, or the current task is interrupted, whichever happens first.
    ///
    /// # Returns
    ///
    /// A future that completes when all async tasks have finished
    fn await_termination(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
