/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Executor
//!
//! Provides trait definitions similar to JDK Executor interface for executing submitted tasks.
//!
//! # Author
//!
//! Haixing Hu

mod async_executor;
mod async_executor_service;
mod callable;
mod direct_executor;
#[allow(clippy::module_inception)]
mod executor;
mod executor_service;
mod runnable;
mod thread_per_task_executor;
mod thread_per_task_executor_service;
mod tokio_executor;
mod tokio_executor_service;

pub use async_executor::AsyncExecutor;
pub use async_executor_service::AsyncExecutorService;
pub use callable::{
    BoxCallable,
    Callable,
};
pub use direct_executor::DirectExecutor;
pub use executor::Executor;
pub use executor_service::ExecutorService;
pub use runnable::{
    BoxRunnable,
    Runnable,
};
pub use thread_per_task_executor::ThreadPerTaskExecutor;
pub use thread_per_task_executor_service::ThreadPerTaskExecutorService;
pub use tokio_executor::TokioExecutor;
pub use tokio_executor_service::TokioExecutorService;
