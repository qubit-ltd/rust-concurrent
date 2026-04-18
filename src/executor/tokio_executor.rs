/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::future::Future;

use super::async_executor::AsyncExecutor;

/// Executes asynchronous tasks by delegating to `tokio::spawn`.
#[derive(Debug, Default, Clone, Copy)]
pub struct TokioExecutor;

impl AsyncExecutor for TokioExecutor {
    /// Spawns the future onto the current Tokio runtime.
    #[inline]
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        std::mem::drop(tokio::spawn(future));
    }
}
