/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 64-bit Floating Point
//!
//! Provides an easy-to-use atomic 64-bit floating point type with sensible
//! default memory orderings. Implemented using bit conversion with AtomicU64.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::atomic::traits::{Atomic, UpdatableAtomic};

/// Atomic 64-bit floating point number.
///
/// Provides easy-to-use atomic operations with automatic memory ordering
/// selection. Implemented using `AtomicU64` with bit conversion.
///
/// # Features
///
/// - Automatic memory ordering selection
/// - Arithmetic operations via CAS loops
/// - Zero-cost abstraction with inline methods
/// - Access to underlying type via `inner()` for advanced use cases
///
/// # Limitations
///
/// - Arithmetic operations use CAS loops (slower than integer operations)
/// - NaN values may cause unexpected behavior in CAS operations
/// - No max/min operations (complex floating point semantics)
///
/// # Example
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicF64;
///
/// let atomic = AtomicF64::new(3.14159);
/// atomic.add(1.0);
/// assert_eq!(atomic.get(), 4.14159);
/// ```
///
/// # Author
///
/// Haixing Hu
#[repr(transparent)]
pub struct AtomicF64 {
    inner: AtomicU64,
}

impl AtomicF64 {
    /// Creates a new atomic floating point number.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF64;
    ///
    /// let atomic = AtomicF64::new(3.14159);
    /// assert_eq!(atomic.get(), 3.14159);
    /// ```
    #[inline]
    pub fn new(value: f64) -> Self {
        Self {
            inner: AtomicU64::new(value.to_bits()),
        }
    }

    /// Gets the current value.
    ///
    /// Uses `Acquire` ordering.
    ///
    /// # Returns
    ///
    /// The current value.
    #[inline]
    pub fn get(&self) -> f64 {
        f64::from_bits(self.inner.load(Ordering::Acquire))
    }

    /// Sets a new value.
    ///
    /// Uses `Release` ordering.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to set.
    #[inline]
    pub fn set(&self, value: f64) {
        self.inner.store(value.to_bits(), Ordering::Release);
    }

    /// Swaps the current value with a new value, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to swap in.
    ///
    /// # Returns
    ///
    /// The old value.
    #[inline]
    pub fn swap(&self, value: f64) -> f64 {
        f64::from_bits(self.inner.swap(value.to_bits(), Ordering::AcqRel))
    }

    /// Compares and sets the value atomically.
    ///
    /// If the current value equals `current`, sets it to `new` and returns
    /// `Ok(())`. Otherwise, returns `Err(actual)` where `actual` is the
    /// current value.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(actual)` on failure.
    #[inline]
    pub fn compare_and_set(&self, current: f64, new: f64) -> Result<(), f64> {
        self.inner
            .compare_exchange(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(f64::from_bits)
    }

    /// Weak version of compare-and-set.
    ///
    /// May spuriously fail even when the comparison succeeds. Should be used
    /// in a loop.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(actual)` on failure.
    #[inline]
    pub fn compare_and_set_weak(&self, current: f64, new: f64) -> Result<(), f64> {
        self.inner
            .compare_exchange_weak(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(f64::from_bits)
    }

    /// Compares and exchanges the value atomically, returning the previous
    /// value.
    ///
    /// If the current value equals `current`, sets it to `new` and returns
    /// the old value. Otherwise, returns the actual current value.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// The value before the operation.
    #[inline]
    pub fn compare_and_exchange(&self, current: f64, new: f64) -> f64 {
        match self.inner.compare_exchange(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f64::from_bits(prev_bits),
            Err(actual_bits) => f64::from_bits(actual_bits),
        }
    }

    /// Weak version of compare-and-exchange.
    ///
    /// May spuriously fail even when the comparison succeeds. Should be used
    /// in a loop.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `current` - The expected current value.
    /// * `new` - The new value to set if current matches.
    ///
    /// # Returns
    ///
    /// The value before the operation.
    #[inline]
    pub fn compare_and_exchange_weak(&self, current: f64, new: f64) -> f64 {
        match self.inner.compare_exchange_weak(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f64::from_bits(prev_bits),
            Err(actual_bits) => f64::from_bits(actual_bits),
        }
    }

    /// Atomically adds a value, returning the new value.
    ///
    /// Internally uses a CAS loop. May be slow in high-contention scenarios.
    ///
    /// # Parameters
    ///
    /// * `delta` - The value to add.
    ///
    /// # Returns
    ///
    /// The new value after adding.
    #[inline]
    pub fn add(&self, delta: f64) -> f64 {
        let mut current = self.get();
        loop {
            let new = current + delta;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomically subtracts a value, returning the new value.
    ///
    /// Internally uses a CAS loop. May be slow in high-contention scenarios.
    ///
    /// # Parameters
    ///
    /// * `delta` - The value to subtract.
    ///
    /// # Returns
    ///
    /// The new value after subtracting.
    #[inline]
    pub fn sub(&self, delta: f64) -> f64 {
        let mut current = self.get();
        loop {
            let new = current - delta;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomically multiplies by a factor, returning the new value.
    ///
    /// Internally uses a CAS loop. May be slow in high-contention scenarios.
    ///
    /// # Parameters
    ///
    /// * `factor` - The factor to multiply by.
    ///
    /// # Returns
    ///
    /// The new value after multiplying.
    #[inline]
    pub fn mul(&self, factor: f64) -> f64 {
        let mut current = self.get();
        loop {
            let new = current * factor;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    /// Atomically divides by a divisor, returning the new value.
    ///
    /// Internally uses a CAS loop. May be slow in high-contention scenarios.
    ///
    /// # Parameters
    ///
    /// * `divisor` - The divisor to divide by.
    ///
    /// # Returns
    ///
    /// The new value after dividing.
    #[inline]
    pub fn div(&self, divisor: f64) -> f64 {
        let mut current = self.get();
        loop {
            let new = current / divisor;
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    /// Updates the value using a function, returning the old value.
    ///
    /// Internally uses a CAS loop until the update succeeds.
    ///
    /// # Parameters
    ///
    /// * `f` - A function that takes the current value and returns the new
    ///   value.
    ///
    /// # Returns
    ///
    /// The old value before the update.
    #[inline]
    pub fn get_and_update<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }

    /// Updates the value using a function, returning the new value.
    ///
    /// Internally uses a CAS loop until the update succeeds.
    ///
    /// # Parameters
    ///
    /// * `f` - A function that takes the current value and returns the new
    ///   value.
    ///
    /// # Returns
    ///
    /// The new value after the update.
    #[inline]
    pub fn update_and_get<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    /// Gets a reference to the underlying standard library atomic type.
    ///
    /// This allows direct access to the standard library's atomic operations
    /// for advanced use cases that require fine-grained control over memory
    /// ordering.
    ///
    /// # Returns
    ///
    /// A reference to the underlying `std::sync::atomic::AtomicU64`.
    #[inline]
    pub fn inner(&self) -> &AtomicU64 {
        &self.inner
    }
}

impl Atomic for AtomicF64 {
    type Value = f64;

    #[inline]
    fn get(&self) -> f64 {
        self.get()
    }

    #[inline]
    fn set(&self, value: f64) {
        self.set(value);
    }

    #[inline]
    fn swap(&self, value: f64) -> f64 {
        self.swap(value)
    }

    #[inline]
    fn compare_and_set(&self, current: f64, new: f64) -> Result<(), f64> {
        self.compare_and_set(current, new)
    }

    #[inline]
    fn compare_and_exchange(&self, current: f64, new: f64) -> f64 {
        self.compare_and_exchange(current, new)
    }
}

impl UpdatableAtomic for AtomicF64 {
    #[inline]
    fn get_and_update<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        self.get_and_update(f)
    }

    #[inline]
    fn update_and_get<F>(&self, f: F) -> f64
    where
        F: Fn(f64) -> f64,
    {
        self.update_and_get(f)
    }
}

unsafe impl Send for AtomicF64 {}
unsafe impl Sync for AtomicF64 {}

impl Default for AtomicF64 {
    #[inline]
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl From<f64> for AtomicF64 {
    #[inline]
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for AtomicF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicF64")
            .field("value", &self.get())
            .finish()
    }
}

impl fmt::Display for AtomicF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}
