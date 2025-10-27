/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Prism3 Concurrent - Concurrency Utilities Library
//!
//! # Author
//!
//! Haixing Hu

pub mod atomic;
pub mod double_checked;
pub mod executor;
pub mod lock;

pub use atomic::{
    Atomic, AtomicBool, AtomicF32, AtomicF64, AtomicI16, AtomicI32, AtomicI64, AtomicI8,
    AtomicInteger, AtomicIsize, AtomicRef, AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
    UpdatableAtomic,
};
pub use double_checked::{
    BuilderError, DoubleCheckedLockExecutor, ExecutionBuilder, ExecutionResult, ExecutorError,
    LogConfig,
};
pub use executor::{
    AsyncExecutor, AsyncExecutorService, Callable, Executor, ExecutorService, Runnable,
};
pub use lock::{ArcAsyncMutex, ArcAsyncRwLock, ArcMutex, ArcRwLock, AsyncLock, Lock};
