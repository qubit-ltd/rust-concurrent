/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
/// Callable task trait that returns a result
///
/// Represents a computation task that can return a result. Similar to the
/// `Callable` interface in JDK.
///
/// Unlike `Runnable`, `Callable` can return a computation result or an error.
///
/// # Type Parameters
///
/// * `T` - The result type returned by the task
/// * `E` - The error type that the task may produce
///
/// # Examples
///
/// ```rust
/// use qubit_concurrent::Callable;
///
/// struct ComputeTask {
///     x: i32,
///     y: i32,
/// }
///
/// impl Callable<i32, String> for ComputeTask {
///     fn call(&self) -> Result<i32, String> {
///         if self.y == 0 {
///             Err(String::from("Divisor cannot be zero"))
///         } else {
///             Ok(self.x / self.y)
///         }
///     }
/// }
///
/// let task = ComputeTask { x: 10, y: 2 };
/// match task.call() {
///     Ok(result) => println!("Computation result: {}", result),
///     Err(e) => println!("Computation error: {}", e),
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub trait Callable<T, E>: Send
where
    T: Send,
    E: Send,
{
    /// Computes a result
    ///
    /// This method will be called when the task is scheduled for execution by
    /// the executor.
    ///
    /// The method can return a computation result or an error.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` - Computation succeeded, returns the result
    /// * `Err(E)` - Computation failed, returns an error
    fn call(&self) -> Result<T, E>;
}
