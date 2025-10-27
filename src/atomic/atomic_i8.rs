/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 8-bit Signed Integer
//!
//! Provides an easy-to-use atomic 8-bit signed integer type with sensible
//! default memory orderings.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::Ordering;

use crate::atomic::atomic_integer_macro::impl_atomic_integer;

impl_atomic_integer!(
    AtomicI8,
    std::sync::atomic::AtomicI8,
    i8,
    "8-bit signed integer"
);
