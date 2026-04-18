/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
/// Synchronous task executor trait
///
/// Provides functionality similar to the JDK `Executor` interface for executing
/// submitted synchronous tasks.
///
/// The executor is responsible for decoupling the execution of tasks from their
/// submission, so that task submitters do not need to worry about how tasks are
/// executed, which thread they run on, etc.
///
/// # Features
///
/// - Decouples task submission and execution: Task submitters only need to
///   submit tasks without worrying about execution details.
/// - Flexible execution strategies: Implementors can freely choose synchronous
///   execution, thread pool execution, and other strategies.
/// - Focuses on synchronous tasks: Only handles regular closure tasks, not
///   async Futures.
///
/// # Use Cases
///
/// - Thread pools: Submit tasks to a thread pool for execution.
/// - Task schedulers: Schedule task execution based on priority, delay, and
///   other policies.
/// - CPU-intensive computation: Execute computationally intensive synchronous
///   tasks.
/// - Testing environment: Use synchronous executors in tests for easier
///   debugging and verification.
///
/// # Examples
///
/// ```rust
/// use qubit_concurrent::Executor;
///
/// struct SimpleExecutor;
///
/// impl Executor for SimpleExecutor {
///     fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
///         // Simple synchronous execution
///         task();
///     }
/// }
///
/// let executor = SimpleExecutor;
/// executor.execute(Box::new(|| {
///     println!("Synchronous task executing");
/// }));
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub trait Executor: Send + Sync {
    /// Executes the given synchronous task
    ///
    /// Submits the task to the executor for execution. The task may be executed
    /// synchronously on the current thread, or asynchronously on another thread,
    /// depending on the implementor's strategy.
    ///
    /// # Parameters
    ///
    /// * `task` - The synchronous task to be executed, which must implement the
    ///   `Send` trait for transfer between threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_concurrent::Executor;
    ///
    /// let executor = create_executor();
    /// executor.execute(Box::new(|| {
    ///     println!("Task is executing");
    /// }));
    /// ```
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>);
}
