/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Double-Checked Lock Executor
//!
//! Provides a double-checked lock executor for executing tasks with condition
//! checking and prepare lifecycle support.
//!
//! # Author
//!
//! Haixing Hu

pub mod builder_error;
pub mod double_checked_lock;
pub mod execution_builder;
pub mod execution_context;
pub mod execution_logger;
pub mod execution_result;
pub mod executor_config;
pub mod executor_error;

pub use builder_error::BuilderError;
pub use double_checked_lock::DoubleCheckedLock;
pub use execution_builder::{Conditioned, Configuring, ExecutionBuilder, Initial};
pub use execution_context::ExecutionContext;
pub use execution_logger::ExecutionLogger;
pub use execution_result::ExecutionResult;
pub use executor_config::ExecutorConfig;
pub use executor_error::ExecutorError;
