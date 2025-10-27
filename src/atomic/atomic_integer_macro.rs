/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic Integer Macro
//!
//! Provides a macro to generate atomic integer types with consistent
//! implementations.
//!
//! # Author
//!
//! Haixing Hu

/// Macro to generate atomic integer types.
///
/// This macro generates a complete atomic integer type with all methods,
/// trait implementations, and documentation.
///
/// # Parameters
///
/// * `$name` - The name of the atomic type (e.g., `AtomicI32`)
/// * `$inner_type` - The underlying std atomic type (e.g.,
///   `std::sync::atomic::AtomicI32`)
/// * `$value_type` - The value type (e.g., `i32`)
/// * `$doc_type` - The type description for documentation (e.g., "32-bit
///   signed integer")
macro_rules! impl_atomic_integer {
    ($name:ident, $inner_type:ty, $value_type:ty, $doc_type:expr) => {
        #[doc = concat!("Atomic ", $doc_type, ".")]
        ///
        /// Provides easy-to-use atomic operations with automatic memory
        /// ordering selection. All methods are thread-safe and can be shared
        /// across threads.
        ///
        /// # Features
        ///
        /// - Automatic memory ordering selection
        /// - Rich set of integer operations (increment, decrement,
        ///   arithmetic, etc.)
        /// - Zero-cost abstraction with inline methods
        /// - Access to underlying type via `inner()` for advanced use cases
        ///
        /// # Example
        ///
        /// ```rust
        #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
        /// use std::sync::Arc;
        /// use std::thread;
        ///
        #[doc = concat!("let counter = Arc::new(", stringify!($name), "::new(0));")]
        /// let mut handles = vec![];
        ///
        /// for _ in 0..10 {
        ///     let counter = counter.clone();
        ///     let handle = thread::spawn(move || {
        ///         for _ in 0..100 {
        ///             counter.increment_and_get();
        ///         }
        ///     });
        ///     handles.push(handle);
        /// }
        ///
        /// for handle in handles {
        ///     handle.join().unwrap();
        /// }
        ///
        /// assert_eq!(counter.get(), 1000);
        /// ```
        ///
        /// # Author
        ///
        /// Haixing Hu
        #[repr(transparent)]
        pub struct $name {
            inner: $inner_type,
        }

        impl $name {
            /// Creates a new atomic integer.
            ///
            /// # Parameters
            ///
            /// * `value` - The initial value.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(42);")]
            /// assert_eq!(atomic.get(), 42);
            /// ```
            #[inline]
            pub const fn new(value: $value_type) -> Self {
                Self {
                    inner: <$inner_type>::new(value),
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(42);")]
            /// assert_eq!(atomic.get(), 42);
            /// ```
            #[inline]
            pub fn get(&self) -> $value_type {
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0);")]
            /// atomic.set(42);
            /// assert_eq!(atomic.get(), 42);
            /// ```
            #[inline]
            pub fn set(&self, value: $value_type) {
                self.inner.store(value, Ordering::Release);
            }

            /// Swaps the current value with a new value, returning the old
            /// value.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.swap(20);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 20);
            /// ```
            #[inline]
            pub fn swap(&self, value: $value_type) -> $value_type {
                self.inner.swap(value, Ordering::AcqRel)
            }

            /// Compares and sets the value atomically.
            ///
            /// If the current value equals `current`, sets it to `new` and
            /// returns `Ok(())`. Otherwise, returns `Err(actual)` where
            /// `actual` is the current value.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// assert!(atomic.compare_and_set(10, 20).is_ok());
            /// assert_eq!(atomic.get(), 20);
            /// ```
            #[inline]
            pub fn compare_and_set(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.inner
                    .compare_exchange(
                        current,
                        new,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .map(|_| ())
            }

            /// Weak version of compare-and-set.
            ///
            /// May spuriously fail even when the comparison succeeds. Should
            /// be used in a loop.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let mut current = atomic.get();
            /// loop {
            ///     match atomic.compare_and_set_weak(current, current + 1) {
            ///         Ok(_) => break,
            ///         Err(actual) => current = actual,
            ///     }
            /// }
            /// assert_eq!(atomic.get(), 11);
            /// ```
            #[inline]
            pub fn compare_and_set_weak(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.inner
                    .compare_exchange_weak(
                        current,
                        new,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .map(|_| ())
            }

            /// Compares and exchanges the value atomically, returning the
            /// previous value.
            ///
            /// If the current value equals `current`, sets it to `new` and
            /// returns the old value. Otherwise, returns the actual current
            /// value.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let prev = atomic.compare_and_exchange(10, 20);
            /// assert_eq!(prev, 10);
            /// assert_eq!(atomic.get(), 20);
            /// ```
            #[inline]
            pub fn compare_and_exchange(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                match self.inner.compare_exchange(
                    current,
                    new,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(prev) => prev,
                    Err(actual) => actual,
                }
            }

            /// Weak version of compare-and-exchange.
            ///
            /// May spuriously fail even when the comparison succeeds. Should
            /// be used in a loop.
            ///
            /// Uses `AcqRel` ordering on success and `Acquire` ordering on
            /// failure.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let mut current = atomic.get();
            /// loop {
            ///     let prev =
            ///         atomic.compare_and_exchange_weak(current, current + 5);
            ///     if prev == current {
            ///         break;
            ///     }
            ///     current = prev;
            /// }
            /// assert_eq!(atomic.get(), 15);
            /// ```
            #[inline]
            pub fn compare_and_exchange_weak(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                match self.inner.compare_exchange_weak(
                    current,
                    new,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(prev) => prev,
                    Err(actual) => actual,
                }
            }

            /// Increments the value by 1, returning the old value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Returns
            ///
            /// The old value before incrementing.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.get_and_increment();
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 11);
            /// ```
            #[inline]
            pub fn get_and_increment(&self) -> $value_type {
                self.inner.fetch_add(1, Ordering::Relaxed)
            }

            /// Increments the value by 1, returning the new value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Returns
            ///
            /// The new value after incrementing.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.increment_and_get();
            /// assert_eq!(new, 11);
            /// ```
            #[inline]
            pub fn increment_and_get(&self) -> $value_type {
                self.inner.fetch_add(1, Ordering::Relaxed) + 1
            }

            /// Decrements the value by 1, returning the old value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Returns
            ///
            /// The old value before decrementing.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.get_and_decrement();
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 9);
            /// ```
            #[inline]
            pub fn get_and_decrement(&self) -> $value_type {
                self.inner.fetch_sub(1, Ordering::Relaxed)
            }

            /// Decrements the value by 1, returning the new value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Returns
            ///
            /// The new value after decrementing.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.decrement_and_get();
            /// assert_eq!(new, 9);
            /// ```
            #[inline]
            pub fn decrement_and_get(&self) -> $value_type {
                self.inner.fetch_sub(1, Ordering::Relaxed) - 1
            }

            /// Adds a delta to the value, returning the old value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Parameters
            ///
            /// * `delta` - The value to add.
            ///
            /// # Returns
            ///
            /// The old value before adding.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.get_and_add(5);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 15);
            /// ```
            #[inline]
            pub fn get_and_add(&self, delta: $value_type) -> $value_type {
                self.inner.fetch_add(delta, Ordering::Relaxed)
            }

            /// Adds a delta to the value, returning the new value.
            ///
            /// Uses `Relaxed` ordering.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.add_and_get(5);
            /// assert_eq!(new, 15);
            /// ```
            #[inline]
            pub fn add_and_get(&self, delta: $value_type) -> $value_type {
                self.inner.fetch_add(delta, Ordering::Relaxed) + delta
            }

            /// Subtracts a delta from the value, returning the old value.
            ///
            /// Uses `Relaxed` ordering.
            ///
            /// # Parameters
            ///
            /// * `delta` - The value to subtract.
            ///
            /// # Returns
            ///
            /// The old value before subtracting.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.get_and_sub(3);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 7);
            /// ```
            #[inline]
            pub fn get_and_sub(&self, delta: $value_type) -> $value_type {
                self.inner.fetch_sub(delta, Ordering::Relaxed)
            }

            /// Subtracts a delta from the value, returning the new value.
            ///
            /// Uses `Relaxed` ordering.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.sub_and_get(3);
            /// assert_eq!(new, 7);
            /// ```
            #[inline]
            pub fn sub_and_get(&self, delta: $value_type) -> $value_type {
                self.inner.fetch_sub(delta, Ordering::Relaxed) - delta
            }

            /// Performs bitwise AND, returning the old value.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1111);")]
            /// let old = atomic.get_and_bitand(0b1100);
            /// assert_eq!(old, 0b1111);
            /// assert_eq!(atomic.get(), 0b1100);
            /// ```
            #[inline]
            pub fn get_and_bitand(&self, value: $value_type) -> $value_type {
                self.inner.fetch_and(value, Ordering::AcqRel)
            }

            /// Performs bitwise OR, returning the old value.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1100);")]
            /// let old = atomic.get_and_bitor(0b0011);
            /// assert_eq!(old, 0b1100);
            /// assert_eq!(atomic.get(), 0b1111);
            /// ```
            #[inline]
            pub fn get_and_bitor(&self, value: $value_type) -> $value_type {
                self.inner.fetch_or(value, Ordering::AcqRel)
            }

            /// Performs bitwise XOR, returning the old value.
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
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0b1100);")]
            /// let old = atomic.get_and_bitxor(0b0110);
            /// assert_eq!(old, 0b1100);
            /// assert_eq!(atomic.get(), 0b1010);
            /// ```
            #[inline]
            pub fn get_and_bitxor(&self, value: $value_type) -> $value_type {
                self.inner.fetch_xor(value, Ordering::AcqRel)
            }

            /// Updates the value using a function, returning the old value.
            ///
            /// Internally uses a CAS loop until the update succeeds.
            ///
            /// # Parameters
            ///
            /// * `f` - A function that takes the current value and returns
            ///   the new value.
            ///
            /// # Returns
            ///
            /// The old value before the update.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.get_and_update(|x| x * 2);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 20);
            /// ```
            #[inline]
            pub fn get_and_update<F>(&self, f: F) -> $value_type
            where
                F: Fn($value_type) -> $value_type,
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
            /// * `f` - A function that takes the current value and returns
            ///   the new value.
            ///
            /// # Returns
            ///
            /// The new value after the update.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.update_and_get(|x| x * 2);
            /// assert_eq!(new, 20);
            /// ```
            #[inline]
            pub fn update_and_get<F>(&self, f: F) -> $value_type
            where
                F: Fn($value_type) -> $value_type,
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

            /// Accumulates a value using a binary function, returning the
            /// old value.
            ///
            /// Internally uses a CAS loop until the update succeeds.
            ///
            /// # Parameters
            ///
            /// * `x` - The value to accumulate with.
            /// * `f` - A binary function that takes the current value and
            ///   `x`, returning the new value.
            ///
            /// # Returns
            ///
            /// The old value before the accumulation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let old = atomic.get_and_accumulate(5, |a, b| a + b);
            /// assert_eq!(old, 10);
            /// assert_eq!(atomic.get(), 15);
            /// ```
            #[inline]
            pub fn get_and_accumulate<F>(
                &self,
                x: $value_type,
                f: F,
            ) -> $value_type
            where
                F: Fn($value_type, $value_type) -> $value_type,
            {
                let mut current = self.get();
                loop {
                    let new = f(current, x);
                    match self.compare_and_set_weak(current, new) {
                        Ok(_) => return current,
                        Err(actual) => current = actual,
                    }
                }
            }

            /// Accumulates a value using a binary function, returning the
            /// new value.
            ///
            /// Internally uses a CAS loop until the update succeeds.
            ///
            /// # Parameters
            ///
            /// * `x` - The value to accumulate with.
            /// * `f` - A binary function that takes the current value and
            ///   `x`, returning the new value.
            ///
            /// # Returns
            ///
            /// The new value after the accumulation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.accumulate_and_get(5, |a, b| a + b);
            /// assert_eq!(new, 15);
            /// ```
            #[inline]
            pub fn accumulate_and_get<F>(
                &self,
                x: $value_type,
                f: F,
            ) -> $value_type
            where
                F: Fn($value_type, $value_type) -> $value_type,
            {
                let mut current = self.get();
                loop {
                    let new = f(current, x);
                    match self.compare_and_set_weak(current, new) {
                        Ok(_) => return new,
                        Err(actual) => current = actual,
                    }
                }
            }

            /// Sets the value to the maximum of the current value and the
            /// given value, returning the old value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to compare with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// atomic.get_and_max(20);
            /// assert_eq!(atomic.get(), 20);
            ///
            /// atomic.get_and_max(15);
            /// assert_eq!(atomic.get(), 20);
            /// ```
            #[inline]
            pub fn get_and_max(&self, value: $value_type) -> $value_type {
                self.inner.fetch_max(value, Ordering::AcqRel)
            }

            /// Sets the value to the maximum of the current value and the
            /// given value, returning the new value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to compare with.
            ///
            /// # Returns
            ///
            /// The new value after the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.max_and_get(20);
            /// assert_eq!(new, 20);
            /// ```
            #[inline]
            pub fn max_and_get(&self, value: $value_type) -> $value_type {
                let old = self.inner.fetch_max(value, Ordering::AcqRel);
                old.max(value)
            }

            /// Sets the value to the minimum of the current value and the
            /// given value, returning the old value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to compare with.
            ///
            /// # Returns
            ///
            /// The old value before the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// atomic.get_and_min(5);
            /// assert_eq!(atomic.get(), 5);
            ///
            /// atomic.get_and_min(8);
            /// assert_eq!(atomic.get(), 5);
            /// ```
            #[inline]
            pub fn get_and_min(&self, value: $value_type) -> $value_type {
                self.inner.fetch_min(value, Ordering::AcqRel)
            }

            /// Sets the value to the minimum of the current value and the
            /// given value, returning the new value.
            ///
            /// Uses `AcqRel` ordering.
            ///
            /// # Parameters
            ///
            /// * `value` - The value to compare with.
            ///
            /// # Returns
            ///
            /// The new value after the operation.
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(10);")]
            /// let new = atomic.min_and_get(5);
            /// assert_eq!(new, 5);
            /// ```
            #[inline]
            pub fn min_and_get(&self, value: $value_type) -> $value_type {
                let old = self.inner.fetch_min(value, Ordering::AcqRel);
                old.min(value)
            }

            /// Gets a reference to the underlying standard library atomic
            /// type.
            ///
            /// This allows direct access to the standard library's atomic
            /// operations for advanced use cases that require fine-grained
            /// control over memory ordering.
            ///
            /// # Returns
            ///
            #[doc = concat!("A reference to the underlying `std::sync::atomic::", stringify!($inner_type), "`.")]
            ///
            /// # Example
            ///
            /// ```rust
            #[doc = concat!("use prism3_rust_concurrent::atomic::", stringify!($name), ";")]
            /// use std::sync::atomic::Ordering;
            ///
            #[doc = concat!("let atomic = ", stringify!($name), "::new(0);")]
            /// atomic.inner().store(42, Ordering::Relaxed);
            /// assert_eq!(atomic.inner().load(Ordering::Relaxed), 42);
            /// ```
            #[inline]
            pub fn inner(&self) -> &$inner_type {
                &self.inner
            }
        }

        impl crate::atomic::traits::Atomic for $name {
            type Value = $value_type;

            #[inline]
            fn get(&self) -> $value_type {
                self.get()
            }

            #[inline]
            fn set(&self, value: $value_type) {
                self.set(value);
            }

            #[inline]
            fn swap(&self, value: $value_type) -> $value_type {
                self.swap(value)
            }

            #[inline]
            fn compare_and_set(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> Result<(), $value_type> {
                self.compare_and_set(current, new)
            }

            #[inline]
            fn compare_and_exchange(
                &self,
                current: $value_type,
                new: $value_type,
            ) -> $value_type {
                self.compare_and_exchange(current, new)
            }
        }

        impl crate::atomic::traits::UpdatableAtomic for $name {
            #[inline]
            fn get_and_update<F>(&self, f: F) -> $value_type
            where
                F: Fn($value_type) -> $value_type,
            {
                self.get_and_update(f)
            }

            #[inline]
            fn update_and_get<F>(&self, f: F) -> $value_type
            where
                F: Fn($value_type) -> $value_type,
            {
                self.update_and_get(f)
            }
        }

        impl crate::atomic::traits::AtomicInteger for $name {
            #[inline]
            fn get_and_increment(&self) -> $value_type {
                self.get_and_increment()
            }

            #[inline]
            fn increment_and_get(&self) -> $value_type {
                self.increment_and_get()
            }

            #[inline]
            fn get_and_decrement(&self) -> $value_type {
                self.get_and_decrement()
            }

            #[inline]
            fn decrement_and_get(&self) -> $value_type {
                self.decrement_and_get()
            }

            #[inline]
            fn get_and_add(&self, delta: $value_type) -> $value_type {
                self.get_and_add(delta)
            }

            #[inline]
            fn add_and_get(&self, delta: $value_type) -> $value_type {
                self.add_and_get(delta)
            }
        }

        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self::new(0)
            }
        }

        impl From<$value_type> for $name {
            #[inline]
            fn from(value: $value_type) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("value", &self.get())
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.get())
            }
        }
    };
}

pub(crate) use impl_atomic_integer;
