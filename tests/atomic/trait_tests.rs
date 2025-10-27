/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_concurrent::atomic::{
    Atomic, AtomicBool, AtomicF32, AtomicF64, AtomicI32, AtomicI64, AtomicI8, AtomicInteger,
    AtomicRef, AtomicU16, AtomicU64, AtomicUsize, UpdatableAtomic,
};
use std::sync::Arc;

// Test that all types implement the Atomic trait correctly
#[test]
fn test_atomic_trait_bool() {
    fn test_atomic<T: Atomic<Value = bool>>(atomic: &T) {
        atomic.set(true);
        assert_eq!(atomic.get(), true);
        let old = atomic.swap(false);
        assert_eq!(old, true);
        assert_eq!(atomic.get(), false);
    }

    let atomic = AtomicBool::new(false);
    test_atomic(&atomic);
}

#[test]
fn test_atomic_trait_integers() {
    fn test_atomic<T: Atomic<Value = i32>>(atomic: &T) {
        atomic.set(42);
        assert_eq!(atomic.get(), 42);
        let old = atomic.swap(100);
        assert_eq!(old, 42);
        assert_eq!(atomic.get(), 100);

        assert!(atomic.compare_and_set(100, 200).is_ok());
        assert_eq!(atomic.get(), 200);

        let prev = atomic.compare_and_exchange(200, 300);
        assert_eq!(prev, 200);
        assert_eq!(atomic.get(), 300);
    }

    let atomic = AtomicI32::new(0);
    test_atomic(&atomic);
}

#[test]
fn test_atomic_trait_floats() {
    fn test_atomic<T: Atomic<Value = f32>>(atomic: &T) {
        atomic.set(3.14);
        assert!((atomic.get() - 3.14).abs() < 1e-6);
        let old = atomic.swap(2.71);
        assert!((old - 3.14).abs() < 1e-6);
    }

    let atomic = AtomicF32::new(0.0);
    test_atomic(&atomic);
}

#[test]
fn test_atomic_trait_ref() {
    fn test_atomic<T: Atomic<Value = Arc<i32>>>(atomic: &T) {
        atomic.set(Arc::new(42));
        assert_eq!(*atomic.get(), 42);
        let old = atomic.swap(Arc::new(100));
        assert_eq!(*old, 42);
    }

    let atomic = AtomicRef::new(Arc::new(0));
    test_atomic(&atomic);
}

// Test UpdatableAtomic trait
#[test]
fn test_updatable_atomic_trait_integers() {
    fn test_updatable<T: UpdatableAtomic<Value = i32>>(atomic: &T) {
        let old = atomic.get_and_update(|x| x + 10);
        assert_eq!(old, 0);
        assert_eq!(atomic.get(), 10);

        let new = atomic.update_and_get(|x| x * 2);
        assert_eq!(new, 20);
        assert_eq!(atomic.get(), 20);
    }

    let atomic = AtomicI32::new(0);
    test_updatable(&atomic);
}

#[test]
fn test_updatable_atomic_trait_floats() {
    fn test_updatable<T: UpdatableAtomic<Value = f64>>(atomic: &T) {
        let old = atomic.get_and_update(|x| x + 10.0);
        assert!((old - 0.0).abs() < 1e-10);
        assert!((atomic.get() - 10.0).abs() < 1e-10);

        let new = atomic.update_and_get(|x| x * 2.0);
        assert!((new - 20.0).abs() < 1e-10);
    }

    let atomic = AtomicF64::new(0.0);
    test_updatable(&atomic);
}

#[test]
fn test_updatable_atomic_trait_ref() {
    fn test_updatable<T: UpdatableAtomic<Value = Arc<i32>>>(atomic: &T) {
        let old = atomic.get_and_update(|x| Arc::new(*x + 10));
        assert_eq!(*old, 0);
        assert_eq!(*atomic.get(), 10);

        let new = atomic.update_and_get(|x| Arc::new(*x * 2));
        assert_eq!(*new, 20);
    }

    let atomic = AtomicRef::new(Arc::new(0));
    test_updatable(&atomic);
}

// Test AtomicInteger trait
#[test]
fn test_atomic_integer_trait_i8() {
    let atomic = AtomicI8::new(0);
    assert_eq!(atomic.increment_and_get(), 1);
    assert_eq!(atomic.get(), 1);

    assert_eq!(atomic.add_and_get(5), 6);
    assert_eq!(atomic.get(), 6);

    assert_eq!(atomic.decrement_and_get(), 5);
    assert_eq!(atomic.get(), 5);

    atomic.set(0b0101);
    atomic.get_and_bitand(0b0011);
    assert_eq!(atomic.get(), 0b0001);
}

#[test]
fn test_atomic_integer_trait_u16() {
    let atomic = AtomicU16::new(0);
    assert_eq!(atomic.increment_and_get(), 1);
    assert_eq!(atomic.add_and_get(10), 11);
    assert_eq!(atomic.get_and_sub(5), 11);
    assert_eq!(atomic.get(), 6);

    atomic.get_and_max(20);
    assert_eq!(atomic.get(), 20);

    atomic.get_and_min(10);
    assert_eq!(atomic.get(), 10);
}

#[test]
fn test_atomic_integer_trait_i32() {
    let atomic = AtomicI32::new(0);
    let new = atomic.accumulate_and_get(5, |a, b| a + b);
    assert_eq!(new, 5);

    let old = atomic.get_and_accumulate(10, |a, b| a * b);
    assert_eq!(old, 5);
    assert_eq!(atomic.get(), 50);
}

#[test]
fn test_atomic_integer_trait_i64() {
    let atomic = AtomicI64::new(0);
    atomic.increment_and_get();
    atomic.add_and_get(99);
    assert_eq!(atomic.get(), 100);

    atomic.get_and_bitor(0b1111);
    assert_eq!(atomic.get() & 0b1111, 0b1111);
}

#[test]
fn test_atomic_integer_trait_usize() {
    let atomic = AtomicUsize::new(0);
    for _ in 0..10 {
        atomic.increment_and_get();
    }
    assert_eq!(atomic.get(), 10);

    atomic.get_and_bitxor(0b1010);
    // Result depends on platform, just check it doesn't panic
    let _ = atomic.get();
}

// Test that all integer types implement all three traits
#[test]
fn test_all_traits_i32() {
    fn test_all<T>(atomic: &T)
    where
        T: Atomic<Value = i32> + UpdatableAtomic<Value = i32> + AtomicInteger<Value = i32>,
    {
        // Atomic trait
        atomic.set(10);
        assert_eq!(atomic.get(), 10);

        // UpdatableAtomic trait
        atomic.update_and_get(|x| x + 5);
        assert_eq!(atomic.get(), 15);

        // AtomicInteger trait
        atomic.increment_and_get();
        assert_eq!(atomic.get(), 16);
    }

    let atomic = AtomicI32::new(0);
    test_all(&atomic);
}

// Test trait object usage
#[test]
fn test_trait_object_atomic() {
    let atomic1: Box<dyn Atomic<Value = i32>> = Box::new(AtomicI32::new(10));
    let atomic2: Box<dyn Atomic<Value = i32>> = Box::new(AtomicI32::new(20));

    assert_eq!(atomic1.get(), 10);
    assert_eq!(atomic2.get(), 20);

    atomic1.set(100);
    atomic2.set(200);

    assert_eq!(atomic1.get(), 100);
    assert_eq!(atomic2.get(), 200);
}

// Test Send and Sync traits
#[test]
fn test_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<AtomicBool>();
    assert_sync::<AtomicBool>();

    assert_send::<AtomicI32>();
    assert_sync::<AtomicI32>();

    assert_send::<AtomicU64>();
    assert_sync::<AtomicU64>();

    assert_send::<AtomicF32>();
    assert_sync::<AtomicF32>();

    assert_send::<AtomicF64>();
    assert_sync::<AtomicF64>();

    assert_send::<AtomicRef<i32>>();
    assert_sync::<AtomicRef<i32>>();
}

// Test Default trait
#[test]
fn test_default_trait() {
    let atomic_bool = AtomicBool::default();
    assert_eq!(atomic_bool.get(), false);

    let atomic_i32 = AtomicI32::default();
    assert_eq!(atomic_i32.get(), 0);

    let atomic_f64 = AtomicF64::default();
    assert_eq!(atomic_f64.get(), 0.0);
}

// Test From trait
#[test]
fn test_from_trait() {
    let atomic_bool = AtomicBool::from(true);
    assert_eq!(atomic_bool.get(), true);

    let atomic_i32 = AtomicI32::from(42);
    assert_eq!(atomic_i32.get(), 42);

    let atomic_f32 = AtomicF32::from(3.14);
    assert!((atomic_f32.get() - 3.14).abs() < 1e-6);
}

// Test Debug and Display traits
#[test]
fn test_debug_display_traits() {
    let atomic_bool = AtomicBool::new(true);
    assert!(format!("{:?}", atomic_bool).contains("true"));
    assert_eq!(format!("{}", atomic_bool), "true");

    let atomic_i32 = AtomicI32::new(42);
    assert!(format!("{:?}", atomic_i32).contains("42"));
    assert_eq!(format!("{}", atomic_i32), "42");

    let atomic_f64 = AtomicF64::new(3.14);
    assert!(format!("{:?}", atomic_f64).contains("3.14"));
    assert!(format!("{}", atomic_f64).contains("3.14"));
}
