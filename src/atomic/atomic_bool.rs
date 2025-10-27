/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic Boolean
//!
//! Provides an easy-to-use atomic boolean type with sensible default memory
//! orderings.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;
use std::sync::atomic::AtomicBool as StdAtomicBool;
use std::sync::atomic::Ordering;

use crate::atomic::traits::Atomic;

/// Atomic boolean type.
///
/// Provides easy-to-use atomic operations with automatic memory ordering
/// selection. All methods are thread-safe and can be shared across threads.
///
/// # Features
///
/// - Automatic memory ordering selection
/// - Rich set of boolean-specific operations
/// - Zero-cost abstraction with inline methods
/// - Access to underlying type via `inner()` for advanced use cases
///
/// # Example
///
/// ```rust
/// use prism3_rust_concurrent::atomic::AtomicBool;
/// use std::sync::Arc;
/// use std::thread;
///
/// let flag = Arc::new(AtomicBool::new(false));
/// let flag_clone = flag.clone();
///
/// let handle = thread::spawn(move || {
///     flag_clone.set(true);
/// });
///
/// handle.join().unwrap();
/// assert_eq!(flag.get(), true);
/// ```
///
/// # Author
///
/// Haixing Hu
#[repr(transparent)]
pub struct AtomicBool {
    inner: StdAtomicBool,
}

impl AtomicBool {
    /// Creates a new atomic boolean.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.get(), false);
    /// ```
    #[inline]
    pub const fn new(value: bool) -> Self {
        Self {
            inner: StdAtomicBool::new(value),
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
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(true);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn get(&self) -> bool {
        self.inner.load(Ordering::Acquire)
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
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// flag.set(true);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn set(&self, value: bool) {
        self.inner.store(value, Ordering::Release);
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
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let old = flag.swap(true);
    /// assert_eq!(old, false);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn swap(&self, value: bool) -> bool {
        self.inner.swap(value, Ordering::AcqRel)
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
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert!(flag.compare_and_set(false, true).is_ok());
    /// assert_eq!(flag.get(), true);
    ///
    /// // Fails because current value is true, not false
    /// assert!(flag.compare_and_set(false, false).is_err());
    /// ```
    #[inline]
    pub fn compare_and_set(&self, current: bool, new: bool) -> Result<(), bool> {
        self.inner
            .compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
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
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let mut current = flag.get();
    /// loop {
    ///     match flag.compare_and_set_weak(current, true) {
    ///         Ok(_) => break,
    ///         Err(actual) => current = actual,
    ///     }
    /// }
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn compare_and_set_weak(&self, current: bool, new: bool) -> Result<(), bool> {
        self.inner
            .compare_exchange_weak(current, new, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
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
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let prev = flag.compare_and_exchange(false, true);
    /// assert_eq!(prev, false);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn compare_and_exchange(&self, current: bool, new: bool) -> bool {
        match self
            .inner
            .compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(prev) => prev,
            Err(actual) => actual,
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
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let mut current = flag.get();
    /// loop {
    ///     let prev = flag.compare_and_exchange_weak(current, true);
    ///     if prev == current {
    ///         break;
    ///     }
    ///     current = prev;
    /// }
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn compare_and_exchange_weak(&self, current: bool, new: bool) -> bool {
        match self
            .inner
            .compare_exchange_weak(current, new, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(prev) => prev,
            Err(actual) => actual,
        }
    }

    /// Atomically sets the value to `true`, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Returns
    ///
    /// The old value before setting to `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let old = flag.get_and_set();
    /// assert_eq!(old, false);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn get_and_set(&self) -> bool {
        self.swap(true)
    }

    /// Atomically sets the value to `true`, returning the new value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Returns
    ///
    /// Always returns `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let new = flag.set_and_get();
    /// assert_eq!(new, true);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn set_and_get(&self) -> bool {
        self.swap(true);
        true
    }

    /// Atomically sets the value to `false`, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Returns
    ///
    /// The old value before setting to `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(true);
    /// let old = flag.get_and_clear();
    /// assert_eq!(old, true);
    /// assert_eq!(flag.get(), false);
    /// ```
    #[inline]
    pub fn get_and_clear(&self) -> bool {
        self.swap(false)
    }

    /// Atomically sets the value to `false`, returning the new value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Returns
    ///
    /// Always returns `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(true);
    /// let new = flag.clear_and_get();
    /// assert_eq!(new, false);
    /// assert_eq!(flag.get(), false);
    /// ```
    #[inline]
    pub fn clear_and_get(&self) -> bool {
        self.swap(false);
        false
    }

    /// Atomically negates the value, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Returns
    ///
    /// The old value before negation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.get_and_negate(), false);
    /// assert_eq!(flag.get(), true);
    /// assert_eq!(flag.get_and_negate(), true);
    /// assert_eq!(flag.get(), false);
    /// ```
    #[inline]
    pub fn get_and_negate(&self) -> bool {
        self.inner.fetch_xor(true, Ordering::AcqRel)
    }

    /// Atomically negates the value, returning the new value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Returns
    ///
    /// The new value after negation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.negate_and_get(), true);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn negate_and_get(&self) -> bool {
        !self.inner.fetch_xor(true, Ordering::AcqRel)
    }

    /// Atomically performs logical AND, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to AND with.
    ///
    /// # Returns
    ///
    /// The old value before the operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(true);
    /// assert_eq!(flag.get_and_logical_and(false), true);
    /// assert_eq!(flag.get(), false);
    /// ```
    #[inline]
    pub fn get_and_logical_and(&self, value: bool) -> bool {
        self.inner.fetch_and(value, Ordering::AcqRel)
    }

    /// Atomically performs logical OR, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to OR with.
    ///
    /// # Returns
    ///
    /// The old value before the operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.get_and_logical_or(true), false);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn get_and_logical_or(&self, value: bool) -> bool {
        self.inner.fetch_or(value, Ordering::AcqRel)
    }

    /// Atomically performs logical XOR, returning the old value.
    ///
    /// Uses `AcqRel` ordering.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to XOR with.
    ///
    /// # Returns
    ///
    /// The old value before the operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.get_and_logical_xor(true), false);
    /// assert_eq!(flag.get(), true);
    /// ```
    #[inline]
    pub fn get_and_logical_xor(&self, value: bool) -> bool {
        self.inner.fetch_xor(value, Ordering::AcqRel)
    }

    /// Conditionally sets the value if it is currently `false`.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `new` - The new value to set if current is `false`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the value was `false` and has been set to `new`,
    /// `Err(true)` if the value was already `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert!(flag.compare_and_set_if_false(true).is_ok());
    /// assert_eq!(flag.get(), true);
    ///
    /// // Second attempt fails
    /// assert!(flag.compare_and_set_if_false(true).is_err());
    /// ```
    #[inline]
    pub fn compare_and_set_if_false(&self, new: bool) -> Result<(), bool> {
        self.compare_and_set(false, new)
    }

    /// Conditionally sets the value if it is currently `true`.
    ///
    /// Uses `AcqRel` ordering on success and `Acquire` ordering on failure.
    ///
    /// # Parameters
    ///
    /// * `new` - The new value to set if current is `true`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the value was `true` and has been set to `new`,
    /// `Err(false)` if the value was already `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    ///
    /// let flag = AtomicBool::new(true);
    /// assert!(flag.compare_and_set_if_true(false).is_ok());
    /// assert_eq!(flag.get(), false);
    ///
    /// // Second attempt fails
    /// assert!(flag.compare_and_set_if_true(false).is_err());
    /// ```
    #[inline]
    pub fn compare_and_set_if_true(&self, new: bool) -> Result<(), bool> {
        self.compare_and_set(true, new)
    }

    /// Gets a reference to the underlying standard library atomic type.
    ///
    /// This allows direct access to the standard library's atomic operations
    /// for advanced use cases that require fine-grained control over memory
    /// ordering.
    ///
    /// # Returns
    ///
    /// A reference to the underlying `std::sync::atomic::AtomicBool`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use prism3_rust_concurrent::atomic::AtomicBool;
    /// use std::sync::atomic::Ordering;
    ///
    /// let flag = AtomicBool::new(false);
    /// flag.inner().store(true, Ordering::Relaxed);
    /// assert_eq!(flag.inner().load(Ordering::Relaxed), true);
    /// ```
    #[inline]
    pub fn inner(&self) -> &StdAtomicBool {
        &self.inner
    }
}

impl Atomic for AtomicBool {
    type Value = bool;

    #[inline]
    fn get(&self) -> bool {
        self.get()
    }

    #[inline]
    fn set(&self, value: bool) {
        self.set(value);
    }

    #[inline]
    fn swap(&self, value: bool) -> bool {
        self.swap(value)
    }

    #[inline]
    fn compare_and_set(&self, current: bool, new: bool) -> Result<(), bool> {
        self.compare_and_set(current, new)
    }

    #[inline]
    fn compare_and_exchange(&self, current: bool, new: bool) -> bool {
        self.compare_and_exchange(current, new)
    }
}

unsafe impl Send for AtomicBool {}
unsafe impl Sync for AtomicBool {}

impl Default for AtomicBool {
    #[inline]
    fn default() -> Self {
        Self::new(false)
    }
}

impl From<bool> for AtomicBool {
    #[inline]
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for AtomicBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicBool")
            .field("value", &self.get())
            .finish()
    }
}

impl fmt::Display for AtomicBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}
