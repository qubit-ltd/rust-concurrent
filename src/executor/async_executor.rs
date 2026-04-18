/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::future::Future;

/// Asynchronous task executor trait
///
/// Provides asynchronous task execution functionality, specifically for executing async Future tasks.
/// Separated from `Executor`, following the single responsibility principle.
///
/// # Features
///
/// - Focuses on asynchronous tasks: Only handles async Futures, not synchronous closures
/// - Non-blocking execution: Tasks execute in an async runtime without blocking threads
/// - Efficient concurrency: Multiple async tasks can share a small number of threads
///
/// # Use Cases
///
/// - Asynchronous I/O operations: Network requests, file I/O, etc.
/// - Concurrent task coordination: Requires large amounts of concurrent async operations
/// - Microservice calls: Asynchronously call multiple services and aggregate results
///
/// # Examples
///
/// ```rust
/// use qubit_concurrent::AsyncExecutor;
///
/// struct SimpleAsyncExecutor;
///
/// impl AsyncExecutor for SimpleAsyncExecutor {
///     fn spawn<F>(&self, future: F)
///     where
///         F: Future<Output = ()> + Send + 'static,
///     {
///         // Spawn task in tokio runtime
///         tokio::spawn(future);
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let executor = SimpleAsyncExecutor;
///     executor.spawn(async {
///         println!("Async task is executing");
///     });
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub trait AsyncExecutor: Send + Sync {
    /// Spawns and executes an asynchronous task
    ///
    /// Submits an async task to the executor for execution. The task will be executed
    /// in the async runtime managed by the executor, without blocking the current thread.
    ///
    /// # Parameters
    ///
    /// * `future` - The async task to be executed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_concurrent::AsyncExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = create_async_executor();
    ///     executor.spawn(async {
    ///         let data = fetch_data().await;
    ///         process_data(data).await;
    ///     });
    /// }
    /// ```
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;
}
