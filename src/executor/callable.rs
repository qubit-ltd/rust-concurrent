/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Callable task abstractions re-exported from `qubit-function`.
//!
//! `Callable<R, E>` is a fallible, one-time, zero-argument computation. It is
//! implemented for closures of type `FnOnce() -> Result<R, E>` and can be boxed
//! with [`Callable::into_box`].

pub use qubit_function::{
    BoxCallable,
    Callable,
};
