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

pub mod lock;

// Re-export main types and functions
pub use lock::{ArcAsyncMutex, ArcAsyncRwLock, ArcMutex, ArcRwLock};
