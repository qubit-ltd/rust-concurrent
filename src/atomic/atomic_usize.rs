/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic Pointer-Sized Unsigned Integer
//!
//! Provides an easy-to-use atomic pointer-sized unsigned integer type with
//! sensible default memory orderings.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::Ordering;

use crate::atomic::atomic_integer_macro::impl_atomic_integer;

impl_atomic_integer!(
    AtomicUsize,
    std::sync::atomic::AtomicUsize,
    usize,
    "pointer-sized unsigned integer"
);
