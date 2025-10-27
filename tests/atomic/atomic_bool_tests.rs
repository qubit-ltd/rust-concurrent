/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_concurrent::atomic::{Atomic, AtomicBool};
use std::sync::Arc;
use std::thread;

#[test]
fn test_new() {
    let atomic = AtomicBool::new(true);
    assert_eq!(atomic.get(), true);
    let atomic = AtomicBool::new(false);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_default() {
    let atomic = AtomicBool::default();
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_from() {
    let atomic = AtomicBool::from(true);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_get_set() {
    let atomic = AtomicBool::new(false);
    atomic.set(true);
    assert_eq!(atomic.get(), true);
    atomic.set(false);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_swap() {
    let atomic = AtomicBool::new(false);
    let old = atomic.swap(true);
    assert_eq!(old, false);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_compare_and_set_success() {
    let atomic = AtomicBool::new(false);
    assert!(atomic.compare_and_set(false, true).is_ok());
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_compare_and_set_failure() {
    let atomic = AtomicBool::new(false);
    match atomic.compare_and_set(true, false) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => assert_eq!(actual, false),
    }
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_compare_and_exchange() {
    let atomic = AtomicBool::new(false);
    let prev = atomic.compare_and_exchange(false, true);
    assert_eq!(prev, false);
    assert_eq!(atomic.get(), true);

    let prev = atomic.compare_and_exchange(false, false);
    assert_eq!(prev, true);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_get_and_set() {
    let atomic = AtomicBool::new(false);
    let old = atomic.get_and_set();
    assert_eq!(old, false);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_set_and_get() {
    let atomic = AtomicBool::new(false);
    let new = atomic.set_and_get();
    assert_eq!(new, true);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_get_and_clear() {
    let atomic = AtomicBool::new(true);
    let old = atomic.get_and_clear();
    assert_eq!(old, true);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_clear_and_get() {
    let atomic = AtomicBool::new(true);
    let new = atomic.clear_and_get();
    assert_eq!(new, false);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_get_and_negate() {
    let atomic = AtomicBool::new(false);
    assert_eq!(atomic.get_and_negate(), false);
    assert_eq!(atomic.get(), true);
    assert_eq!(atomic.get_and_negate(), true);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_negate_and_get() {
    let atomic = AtomicBool::new(false);
    assert_eq!(atomic.negate_and_get(), true);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_get_and_logical_and() {
    let atomic = AtomicBool::new(true);
    assert_eq!(atomic.get_and_logical_and(false), true);
    assert_eq!(atomic.get(), false);

    atomic.set(true);
    assert_eq!(atomic.get_and_logical_and(true), true);
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_get_and_logical_or() {
    let atomic = AtomicBool::new(false);
    assert_eq!(atomic.get_and_logical_or(true), false);
    assert_eq!(atomic.get(), true);

    atomic.set(false);
    assert_eq!(atomic.get_and_logical_or(false), false);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_get_and_logical_xor() {
    let atomic = AtomicBool::new(false);
    assert_eq!(atomic.get_and_logical_xor(true), false);
    assert_eq!(atomic.get(), true);

    assert_eq!(atomic.get_and_logical_xor(true), true);
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_compare_and_set_if_false() {
    let atomic = AtomicBool::new(false);
    assert!(atomic.compare_and_set_if_false(true).is_ok());
    assert_eq!(atomic.get(), true);

    assert!(atomic.compare_and_set_if_false(false).is_err());
    assert_eq!(atomic.get(), true);
}

#[test]
fn test_compare_and_set_if_true() {
    let atomic = AtomicBool::new(true);
    assert!(atomic.compare_and_set_if_true(false).is_ok());
    assert_eq!(atomic.get(), false);

    assert!(atomic.compare_and_set_if_true(true).is_err());
    assert_eq!(atomic.get(), false);
}

#[test]
fn test_concurrent_toggle() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];

    for _ in 0..10 {
        let flag = flag.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                flag.get_and_negate();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // After 1000 toggles, should be false (even number)
    assert_eq!(flag.get(), false);
}

#[test]
fn test_concurrent_set_once() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];
    let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    for _ in 0..10 {
        let flag = flag.clone();
        let success_count = success_count.clone();
        let handle = thread::spawn(move || {
            if flag.compare_and_set_if_false(true).is_ok() {
                success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Only one thread should succeed
    assert_eq!(flag.get(), true);
    assert_eq!(success_count.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn test_trait_atomic() {
    fn test_atomic<T: Atomic<Value = bool>>(atomic: &T) {
        atomic.set(true);
        assert_eq!(atomic.get(), true);
        let old = atomic.swap(false);
        assert_eq!(old, true);
    }

    let atomic = AtomicBool::new(false);
    test_atomic(&atomic);
}

#[test]
fn test_debug_display() {
    let atomic = AtomicBool::new(true);
    let debug_str = format!("{:?}", atomic);
    assert!(debug_str.contains("true"));
    let display_str = format!("{}", atomic);
    assert_eq!(display_str, "true");
}
