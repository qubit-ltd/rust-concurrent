/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic Traits
//!
//! Defines common traits for atomic types, providing a unified interface
//! for atomic operations.
//!
//! # Author
//!
//! Haixing Hu

/// Common trait for all atomic types.
///
/// Provides basic atomic operations including get, set, swap, and
/// compare-and-set.
///
/// # Author
///
/// Haixing Hu
pub trait Atomic {
    /// The value type stored in the atomic.
    type Value;

    /// Gets the current value.
    ///
    /// Uses `Acquire` ordering by default.
    ///
    /// # Returns
    ///
    /// The current value.
    fn get(&self) -> Self::Value;

    /// Sets a new value.
    ///
    /// Uses `Release` ordering by default.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to set.
    fn set(&self, value: Self::Value);

    /// Swaps the current value with a new value, returning the old value.
    ///
    /// Uses `AcqRel` ordering by default.
    ///
    /// # Parameters
    ///
    /// * `value` - The new value to swap in.
    ///
    /// # Returns
    ///
    /// The old value.
    fn swap(&self, value: Self::Value) -> Self::Value;

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
    /// `Ok(())` on success, or `Err(actual)` on failure where `actual` is
    /// the real current value.
    fn compare_and_set(&self, current: Self::Value, new: Self::Value) -> Result<(), Self::Value>;

    /// Compares and exchanges the value atomically, returning the previous
    /// value.
    ///
    /// If the current value equals `current`, sets it to `new` and returns
    /// the old value. Otherwise, returns the actual current value.
    ///
    /// This is similar to `compare_and_set` but always returns the actual
    /// value instead of a Result, which can be more convenient in CAS loops.
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
    /// The value before the operation. If it equals `current`, the
    /// operation succeeded.
    fn compare_and_exchange(&self, current: Self::Value, new: Self::Value) -> Self::Value;
}

/// Trait for atomic types that support functional updates.
///
/// Provides methods to update atomic values using closures.
///
/// # Author
///
/// Haixing Hu
pub trait UpdatableAtomic: Atomic {
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
    fn get_and_update<F>(&self, f: F) -> Self::Value
    where
        F: Fn(Self::Value) -> Self::Value;

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
    fn update_and_get<F>(&self, f: F) -> Self::Value
    where
        F: Fn(Self::Value) -> Self::Value;
}

/// Trait for atomic integer types.
///
/// Provides integer-specific operations like increment, decrement, and
/// arithmetic operations.
///
/// # Author
///
/// Haixing Hu
pub trait AtomicInteger: UpdatableAtomic {
    /// Increments the value by 1, returning the old value.
    ///
    /// Uses `Relaxed` ordering by default.
    ///
    /// # Returns
    ///
    /// The old value before incrementing.
    fn get_and_increment(&self) -> Self::Value;

    /// Increments the value by 1, returning the new value.
    ///
    /// Uses `Relaxed` ordering by default.
    ///
    /// # Returns
    ///
    /// The new value after incrementing.
    fn increment_and_get(&self) -> Self::Value;

    /// Decrements the value by 1, returning the old value.
    ///
    /// Uses `Relaxed` ordering by default.
    ///
    /// # Returns
    ///
    /// The old value before decrementing.
    fn get_and_decrement(&self) -> Self::Value;

    /// Decrements the value by 1, returning the new value.
    ///
    /// Uses `Relaxed` ordering by default.
    ///
    /// # Returns
    ///
    /// The new value after decrementing.
    fn decrement_and_get(&self) -> Self::Value;

    /// Adds a delta to the value, returning the old value.
    ///
    /// Uses `Relaxed` ordering by default.
    ///
    /// # Parameters
    ///
    /// * `delta` - The value to add.
    ///
    /// # Returns
    ///
    /// The old value before adding.
    fn get_and_add(&self, delta: Self::Value) -> Self::Value;

    /// Adds a delta to the value, returning the new value.
    ///
    /// Uses `Relaxed` ordering by default.
    ///
    /// # Parameters
    ///
    /// * `delta` - The value to add.
    ///
    /// # Returns
    ///
    /// The new value after adding.
    fn add_and_get(&self, delta: Self::Value) -> Self::Value;
}
