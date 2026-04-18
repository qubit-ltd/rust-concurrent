/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::thread;

use super::executor::Executor;

/// Executes each submitted synchronous task on a dedicated thread.
///
/// This executor does not track task lifecycle and does not support shutdown.
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadPerTaskExecutor;

impl Executor for ThreadPerTaskExecutor {
    /// Spawns one OS thread per task and detaches it immediately.
    #[inline]
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        let _ = thread::spawn(move || {
            task();
        });
    }
}
