/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
/// Runnable task trait
///
/// Represents a task that can be executed. Similar to the `Runnable` interface in JDK.
///
/// # Examples
///
/// ```rust
/// use qubit_concurrent::Runnable;
///
/// struct MyTask {
///     name: String,
/// }
///
/// impl Runnable for MyTask {
///     fn run(&self) {
///         println!("Task {} is executing", self.name);
///     }
/// }
///
/// let task = MyTask {
///     name: String::from("task-1"),
/// };
/// task.run();
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub trait Runnable: Send {
    /// Executes the task
    ///
    /// This method will be called when the task is scheduled for execution by
    /// the executor.
    fn run(&self);
}
