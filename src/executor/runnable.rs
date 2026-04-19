/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Runnable task abstractions re-exported from `qubit-function`.
//!
//! `Runnable<E>` is a fallible, one-time, zero-argument action. It is
//! implemented for closures of type `FnOnce() -> Result<(), E>` and can be
//! boxed with [`Runnable::into_box`].

pub use qubit_function::{
    BoxRunnable,
    Runnable,
};
