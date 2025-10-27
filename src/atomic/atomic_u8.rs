/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 8-bit Unsigned Integer
//!
//! Provides an easy-to-use atomic 8-bit unsigned integer type with sensible
//! default memory orderings.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::Ordering;

use crate::atomic::atomic_integer_macro::impl_atomic_integer;

impl_atomic_integer!(
    AtomicU8,
    std::sync::atomic::AtomicU8,
    u8,
    "8-bit unsigned integer"
);
