/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Double-Checked Lock Executor
//!
//! Provides a double-checked lock executor for executing tasks with condition
//! checking and rollback support.
//!
//! # Author
//!
//! Haixing Hu

pub mod builder;
pub mod config;
pub mod error;
pub mod double_checked_lock;
pub mod result;
pub mod states;

pub use builder::ExecutionBuilder;
pub use config::{ExecutorConfig, LogConfig};
pub use error::{BuilderError, ExecutorError};
pub use double_checked_lock::DoubleCheckedLock;
pub use result::{ExecutionContext, ExecutionResult};
pub use states::{Conditioned, Configuring, Initial};
