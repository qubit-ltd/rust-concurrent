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
    task::{
        Context,
        Poll,
    },
};

use tokio::task::{
    JoinError,
    JoinHandle,
};

/// Future-backed execution returned by [`TokioExecutor`](super::TokioExecutor).
///
/// This struct **implements [`Future`](std::future::Future)**:
/// [`Output`](std::future::Future::Output) is [`Result<R, E>`](Result) — the
/// success value `R` or the callable's error `E`. Await this type (on a
/// Tokio-driven async context) to receive that result; until then the underlying
/// blocking task may still be running.
///
/// Tokio join errors are not part of that `Result`: task panics are resumed and
/// cancellations panic when the future is awaited.
///
/// # Type Parameters
///
/// * `R` - The task success value.
/// * `E` - The task error value.
///
/// # Author
///
/// Haixing Hu
pub struct TokioExecution<R, E> {
    handle: JoinHandle<Result<R, E>>,
}

impl<R, E> TokioExecution<R, E> {
    /// Creates a Tokio execution wrapper.
    ///
    /// # Parameters
    ///
    /// * `handle` - The Tokio join handle that produces the task result.
    ///
    /// # Returns
    ///
    /// A future-backed execution wrapper.
    #[inline]
    pub(crate) fn new(handle: JoinHandle<Result<R, E>>) -> Self {
        Self { handle }
    }

    /// Returns whether the Tokio task has finished.
    ///
    /// # Returns
    ///
    /// `true` if the underlying Tokio task has completed.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    /// Requests cancellation of the underlying Tokio task.
    ///
    /// Tokio can cancel a blocking task only before it starts. If the blocking
    /// closure is already running, this request is best-effort and awaiting the
    /// execution will still wait for the closure to finish.
    ///
    /// # Returns
    ///
    /// `true` after the cancellation request has been sent to Tokio.
    #[inline]
    pub fn cancel(&self) -> bool {
        self.handle.abort();
        true
    }
}

impl<R, E> Future for TokioExecution<R, E> {
    type Output = Result<R, E>;

    /// Polls the underlying Tokio task.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match Pin::new(&mut this.handle).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(error)) => handle_join_error(error),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Converts a Tokio join error into this execution's panic behavior.
fn handle_join_error<R, E>(error: JoinError) -> Poll<Result<R, E>> {
    if error.is_panic() {
        std::panic::resume_unwind(error.into_panic());
    }
    panic!("tokio execution was cancelled before completion");
}
