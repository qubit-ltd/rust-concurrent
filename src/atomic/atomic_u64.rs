/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 64-bit Unsigned Integer
//!
//! Provides an easy-to-use atomic 64-bit unsigned integer type with sensible
//! default memory orderings.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::Ordering;

use crate::atomic::atomic_integer_macro::impl_atomic_integer;

impl_atomic_integer!(
    AtomicU64,
    std::sync::atomic::AtomicU64,
    u64,
    "64-bit unsigned integer"
);
