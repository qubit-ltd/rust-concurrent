/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_concurrent::atomic::{Atomic, AtomicF64, UpdatableAtomic};
use std::sync::Arc;
use std::thread;

const EPSILON: f64 = 1e-10;

#[test]
fn test_new() {
    let atomic = AtomicF64::new(3.14159);
    assert!((atomic.get() - 3.14159).abs() < EPSILON);
}

#[test]
fn test_default() {
    let atomic = AtomicF64::default();
    assert_eq!(atomic.get(), 0.0);
}

#[test]
fn test_from() {
    let atomic = AtomicF64::from(2.71828);
    assert!((atomic.get() - 2.71828).abs() < EPSILON);
}

#[test]
fn test_get_set() {
    let atomic = AtomicF64::new(0.0);
    atomic.set(3.14159);
    assert!((atomic.get() - 3.14159).abs() < EPSILON);
    atomic.set(-2.5);
    assert!((atomic.get() - (-2.5)).abs() < EPSILON);
}

#[test]
fn test_swap() {
    let atomic = AtomicF64::new(1.0);
    let old = atomic.swap(2.0);
    assert!((old - 1.0).abs() < EPSILON);
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_set_success() {
    let atomic = AtomicF64::new(1.0);
    assert!(atomic.compare_and_set(1.0, 2.0).is_ok());
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_set_failure() {
    let atomic = AtomicF64::new(1.0);
    match atomic.compare_and_set(1.5, 2.0) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert!((actual - 1.0).abs() < EPSILON),
    }
    assert!((atomic.get() - 1.0).abs() < EPSILON);
}

#[test]
fn test_compare_and_exchange() {
    let atomic = AtomicF64::new(1.0);
    let prev = atomic.compare_and_exchange(1.0, 2.0);
    assert!((prev - 1.0).abs() < EPSILON);
    assert!((atomic.get() - 2.0).abs() < EPSILON);
}

#[test]
fn test_add() {
    let atomic = AtomicF64::new(10.0);
    let new = atomic.add(5.5);
    assert!((new - 15.5).abs() < EPSILON);
    assert!((atomic.get() - 15.5).abs() < EPSILON);
}

#[test]
fn test_sub() {
    let atomic = AtomicF64::new(10.0);
    let new = atomic.sub(3.5);
    assert!((new - 6.5).abs() < EPSILON);
    assert!((atomic.get() - 6.5).abs() < EPSILON);
}

#[test]
fn test_mul() {
    let atomic = AtomicF64::new(10.0);
    let new = atomic.mul(2.5);
    assert!((new - 25.0).abs() < EPSILON);
    assert!((atomic.get() - 25.0).abs() < EPSILON);
}

#[test]
fn test_div() {
    let atomic = AtomicF64::new(10.0);
    let new = atomic.div(2.0);
    assert!((new - 5.0).abs() < EPSILON);
    assert!((atomic.get() - 5.0).abs() < EPSILON);
}

#[test]
fn test_get_and_update() {
    let atomic = AtomicF64::new(10.0);
    let old = atomic.get_and_update(|x| x * 2.0);
    assert!((old - 10.0).abs() < EPSILON);
    assert!((atomic.get() - 20.0).abs() < EPSILON);
}

#[test]
fn test_update_and_get() {
    let atomic = AtomicF64::new(10.0);
    let new = atomic.update_and_get(|x| x * 2.0);
    assert!((new - 20.0).abs() < EPSILON);
    assert!((atomic.get() - 20.0).abs() < EPSILON);
}

#[test]
fn test_concurrent_add() {
    let sum = Arc::new(AtomicF64::new(0.0));
    let mut handles = vec![];

    for _ in 0..10 {
        let sum = sum.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                sum.add(0.01);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Due to floating point precision, result may not be exactly 10.0
    let result = sum.get();
    assert!((result - 10.0).abs() < 0.01);
}

#[test]
fn test_trait_atomic() {
    fn test_atomic<T: Atomic<Value = f64>>(atomic: &T) {
        atomic.set(3.14159);
        assert!((atomic.get() - 3.14159).abs() < EPSILON);
        let old = atomic.swap(2.71828);
        assert!((old - 3.14159).abs() < EPSILON);
    }

    let atomic = AtomicF64::new(0.0);
    test_atomic(&atomic);
}

#[test]
fn test_trait_updatable_atomic() {
    fn test_updatable<T: UpdatableAtomic<Value = f64>>(atomic: &T) {
        let new = atomic.update_and_get(|x| x + 10.0);
        assert!((new - 10.0).abs() < EPSILON);
    }

    let atomic = AtomicF64::new(0.0);
    test_updatable(&atomic);
}

#[test]
fn test_debug_display() {
    let atomic = AtomicF64::new(3.14159);
    let debug_str = format!("{:?}", atomic);
    assert!(debug_str.contains("3.14159"));
    let display_str = format!("{}", atomic);
    assert!(display_str.contains("3.14159"));
}

#[test]
fn test_negative_values() {
    let atomic = AtomicF64::new(-10.5);
    assert!((atomic.get() - (-10.5)).abs() < EPSILON);
    atomic.add(5.5);
    assert!((atomic.get() - (-5.0)).abs() < EPSILON);
}

#[test]
fn test_zero() {
    let atomic = AtomicF64::new(0.0);
    assert_eq!(atomic.get(), 0.0);
    atomic.add(1.0);
    assert!((atomic.get() - 1.0).abs() < EPSILON);
}

#[test]
fn test_infinity() {
    let atomic = AtomicF64::new(f64::INFINITY);
    assert_eq!(atomic.get(), f64::INFINITY);
    atomic.set(f64::NEG_INFINITY);
    assert_eq!(atomic.get(), f64::NEG_INFINITY);
}

#[test]
fn test_high_precision() {
    let atomic = AtomicF64::new(1.23456789012345);
    assert!((atomic.get() - 1.23456789012345).abs() < EPSILON);
}
