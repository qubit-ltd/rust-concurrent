/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic 32-bit Floating Point
//!
//! Provides an easy-to-use atomic 32-bit floating point type with sensible
//! default memory orderings. Implemented using bit conversion with AtomicU32.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

use crate::atomic::traits::{Atomic, UpdatableAtomic};

/// Atomic 32-bit floating point number.
///
/// Provides easy-to-use atomic operations with automatic memory ordering
/// selection. Implemented using `AtomicU32` with bit conversion.
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
/// use prism3_rust_concurrent::atomic::AtomicF32;
/// use std::sync::Arc;
/// use std::thread;
///
/// let sum = Arc::new(AtomicF32::new(0.0));
/// let mut handles = vec![];
///
/// for _ in 0..10 {
///     let sum = sum.clone();
///     let handle = thread::spawn(move || {
///         for _ in 0..100 {
///             sum.add(0.1);
///         }
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
///
/// // Note: Due to floating point precision, result may not be exactly 100.0
/// let result = sum.get();
/// assert!((result - 100.0).abs() < 0.01);
/// ```
///
/// # Author
///
/// Haixing Hu
#[repr(transparent)]
pub struct AtomicF32 {
    inner: AtomicU32,
}

impl AtomicF32 {
    /// Creates a new atomic floating point number.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(3.14);
    /// assert_eq!(atomic.get(), 3.14);
    /// ```
    #[inline]
    pub fn new(value: f32) -> Self {
        Self {
            inner: AtomicU32::new(value.to_bits()),
        }
    }

    /// Gets the current value.
    ///
    /// Uses `Acquire` ordering.
    ///
    /// # Returns
    ///
    /// The current value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(3.14);
    /// assert_eq!(atomic.get(), 3.14);
    /// ```
    #[inline]
    pub fn get(&self) -> f32 {
        f32::from_bits(self.inner.load(Ordering::Acquire))
    }

    /// Sets a new value.
    ///
    /// Uses `Release` ordering.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(0.0);
    /// atomic.set(3.14);
    /// assert_eq!(atomic.get(), 3.14);
    /// ```
    #[inline]
    pub fn set(&self, value: f32) {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(1.0);
    /// let old = atomic.swap(2.0);
    /// assert_eq!(old, 1.0);
    /// assert_eq!(atomic.get(), 2.0);
    /// ```
    #[inline]
    pub fn swap(&self, value: f32) -> f32 {
        f32::from_bits(self.inner.swap(value.to_bits(), Ordering::AcqRel))
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
    ///
    /// # Warning
    ///
    /// Due to NaN != NaN, CAS operations with NaN values may behave
    /// unexpectedly. Avoid using NaN in atomic floating point operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(1.0);
    /// assert!(atomic.compare_and_set(1.0, 2.0).is_ok());
    /// assert_eq!(atomic.get(), 2.0);
    /// ```
    #[inline]
    pub fn compare_and_set(&self, current: f32, new: f32) -> Result<(), f32> {
        self.inner
            .compare_exchange(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(f32::from_bits)
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(1.0);
    /// let mut current = atomic.get();
    /// loop {
    ///     match atomic.compare_and_set_weak(current, current + 1.0) {
    ///         Ok(_) => break,
    ///         Err(actual) => current = actual,
    ///     }
    /// }
    /// assert_eq!(atomic.get(), 2.0);
    /// ```
    #[inline]
    pub fn compare_and_set_weak(&self, current: f32, new: f32) -> Result<(), f32> {
        self.inner
            .compare_exchange_weak(
                current.to_bits(),
                new.to_bits(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(f32::from_bits)
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(1.0);
    /// let prev = atomic.compare_and_exchange(1.0, 2.0);
    /// assert_eq!(prev, 1.0);
    /// assert_eq!(atomic.get(), 2.0);
    /// ```
    #[inline]
    pub fn compare_and_exchange(&self, current: f32, new: f32) -> f32 {
        match self.inner.compare_exchange(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f32::from_bits(prev_bits),
            Err(actual_bits) => f32::from_bits(actual_bits),
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(1.0);
    /// let mut current = atomic.get();
    /// loop {
    ///     let prev = atomic.compare_and_exchange_weak(current, current + 1.0);
    ///     if prev == current {
    ///         break;
    ///     }
    ///     current = prev;
    /// }
    /// assert_eq!(atomic.get(), 2.0);
    /// ```
    #[inline]
    pub fn compare_and_exchange_weak(&self, current: f32, new: f32) -> f32 {
        match self.inner.compare_exchange_weak(
            current.to_bits(),
            new.to_bits(),
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(prev_bits) => f32::from_bits(prev_bits),
            Err(actual_bits) => f32::from_bits(actual_bits),
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let new = atomic.add(5.5);
    /// assert_eq!(new, 15.5);
    /// ```
    #[inline]
    pub fn add(&self, delta: f32) -> f32 {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let new = atomic.sub(3.5);
    /// assert_eq!(new, 6.5);
    /// ```
    #[inline]
    pub fn sub(&self, delta: f32) -> f32 {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let new = atomic.mul(2.5);
    /// assert_eq!(new, 25.0);
    /// ```
    #[inline]
    pub fn mul(&self, factor: f32) -> f32 {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let new = atomic.div(2.0);
    /// assert_eq!(new, 5.0);
    /// ```
    #[inline]
    pub fn div(&self, divisor: f32) -> f32 {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let old = atomic.get_and_update(|x| x * 2.0);
    /// assert_eq!(old, 10.0);
    /// assert_eq!(atomic.get(), 20.0);
    /// ```
    #[inline]
    pub fn get_and_update<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32,
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    ///
    /// let atomic = AtomicF32::new(10.0);
    /// let new = atomic.update_and_get(|x| x * 2.0);
    /// assert_eq!(new, 20.0);
    /// ```
    #[inline]
    pub fn update_and_get<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32,
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
    /// A reference to the underlying `std::sync::atomic::AtomicU32`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicF32;
    /// use std::sync::atomic::Ordering;
    ///
    /// let atomic = AtomicF32::new(0.0);
    /// atomic.inner().store(3.14_f32.to_bits(), Ordering::Relaxed);
    /// let bits = atomic.inner().load(Ordering::Relaxed);
    /// assert_eq!(f32::from_bits(bits), 3.14);
    /// ```
    #[inline]
    pub fn inner(&self) -> &AtomicU32 {
        &self.inner
    }
}

impl Atomic for AtomicF32 {
    type Value = f32;

    #[inline]
    fn get(&self) -> f32 {
        self.get()
    }

    #[inline]
    fn set(&self, value: f32) {
        self.set(value);
    }

    #[inline]
    fn swap(&self, value: f32) -> f32 {
        self.swap(value)
    }

    #[inline]
    fn compare_and_set(&self, current: f32, new: f32) -> Result<(), f32> {
        self.compare_and_set(current, new)
    }

    #[inline]
    fn compare_and_exchange(&self, current: f32, new: f32) -> f32 {
        self.compare_and_exchange(current, new)
    }
}

impl UpdatableAtomic for AtomicF32 {
    #[inline]
    fn get_and_update<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32,
    {
        self.get_and_update(f)
    }

    #[inline]
    fn update_and_get<F>(&self, f: F) -> f32
    where
        F: Fn(f32) -> f32,
    {
        self.update_and_get(f)
    }
}

unsafe impl Send for AtomicF32 {}
unsafe impl Sync for AtomicF32 {}

impl Default for AtomicF32 {
    #[inline]
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl From<f32> for AtomicF32 {
    #[inline]
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for AtomicF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicF32")
            .field("value", &self.get())
            .finish()
    }
}

impl fmt::Display for AtomicF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}
