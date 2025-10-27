/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_concurrent::atomic::{Atomic, AtomicRef, UpdatableAtomic};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, PartialEq)]
struct TestData {
    value: i32,
    name: String,
}

#[test]
fn test_new() {
    let data = Arc::new(TestData {
        value: 42,
        name: "test".to_string(),
    });
    let atomic = AtomicRef::new(data.clone());
    assert_eq!(atomic.get().value, 42);
    assert_eq!(atomic.get().name, "test");
}

#[test]
fn test_get_set() {
    let data1 = Arc::new(TestData {
        value: 42,
        name: "first".to_string(),
    });
    let atomic = AtomicRef::new(data1);

    let data2 = Arc::new(TestData {
        value: 100,
        name: "second".to_string(),
    });
    atomic.set(data2);

    let current = atomic.get();
    assert_eq!(current.value, 100);
    assert_eq!(current.name, "second");
}

#[test]
fn test_swap() {
    let data1 = Arc::new(TestData {
        value: 42,
        name: "first".to_string(),
    });
    let atomic = AtomicRef::new(data1.clone());

    let data2 = Arc::new(TestData {
        value: 100,
        name: "second".to_string(),
    });
    let old = atomic.swap(data2);

    assert_eq!(old.value, 42);
    assert_eq!(old.name, "first");
    assert_eq!(atomic.get().value, 100);
}

#[test]
fn test_compare_and_set_success() {
    let data1 = Arc::new(TestData {
        value: 42,
        name: "first".to_string(),
    });
    let atomic = AtomicRef::new(data1.clone());

    let data2 = Arc::new(TestData {
        value: 100,
        name: "second".to_string(),
    });

    let current = atomic.get();
    assert!(atomic.compare_and_set(&current, data2).is_ok());
    assert_eq!(atomic.get().value, 100);
}

#[test]
fn test_compare_and_set_failure() {
    let data1 = Arc::new(TestData {
        value: 42,
        name: "first".to_string(),
    });
    let atomic = AtomicRef::new(data1.clone());

    let data2 = Arc::new(TestData {
        value: 100,
        name: "second".to_string(),
    });

    let wrong_ref = Arc::new(TestData {
        value: 999,
        name: "wrong".to_string(),
    });

    match atomic.compare_and_set(&wrong_ref, data2) {
        Ok(_) => panic!("Should fail"),
        Err(actual) => {
            assert_eq!(actual.value, 42);
            assert_eq!(actual.name, "first");
        }
    }
    assert_eq!(atomic.get().value, 42);
}

#[test]
fn test_compare_and_exchange() {
    let data1 = Arc::new(TestData {
        value: 42,
        name: "first".to_string(),
    });
    let atomic = AtomicRef::new(data1.clone());

    let data2 = Arc::new(TestData {
        value: 100,
        name: "second".to_string(),
    });

    let current = atomic.get();
    let prev = atomic.compare_and_exchange(&current, data2);
    assert!(Arc::ptr_eq(&prev, &current));
    assert_eq!(atomic.get().value, 100);
}

#[test]
fn test_get_and_update() {
    let data = Arc::new(TestData {
        value: 42,
        name: "test".to_string(),
    });
    let atomic = AtomicRef::new(data);

    let old = atomic.get_and_update(|current| {
        Arc::new(TestData {
            value: current.value * 2,
            name: format!("{}_updated", current.name),
        })
    });

    assert_eq!(old.value, 42);
    assert_eq!(old.name, "test");
    assert_eq!(atomic.get().value, 84);
    assert_eq!(atomic.get().name, "test_updated");
}

#[test]
fn test_update_and_get() {
    let data = Arc::new(TestData {
        value: 42,
        name: "test".to_string(),
    });
    let atomic = AtomicRef::new(data);

    let new = atomic.update_and_get(|current| {
        Arc::new(TestData {
            value: current.value * 2,
            name: format!("{}_updated", current.name),
        })
    });

    assert_eq!(new.value, 84);
    assert_eq!(new.name, "test_updated");
    assert_eq!(atomic.get().value, 84);
}

#[test]
fn test_concurrent_updates() {
    let data = Arc::new(TestData {
        value: 0,
        name: "counter".to_string(),
    });
    let atomic = Arc::new(AtomicRef::new(data));
    let mut handles = vec![];

    for i in 0..10 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            atomic.update_and_get(|current| {
                Arc::new(TestData {
                    value: current.value + 1,
                    name: format!("thread_{}", i),
                })
            });
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(atomic.get().value, 10);
}

#[test]
fn test_concurrent_cas() {
    let data = Arc::new(0);
    let atomic = Arc::new(AtomicRef::new(data));
    let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let atomic = atomic.clone();
        let success_count = success_count.clone();
        let handle = thread::spawn(move || {
            let mut current = atomic.get();
            loop {
                let new = Arc::new(*current + 1);
                match atomic.compare_and_set_weak(&current, new) {
                    Ok(_) => {
                        success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        break;
                    }
                    Err(actual) => current = actual,
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*atomic.get(), 10);
    assert_eq!(success_count.load(std::sync::atomic::Ordering::Relaxed), 10);
}

#[test]
fn test_clone() {
    let data = Arc::new(TestData {
        value: 42,
        name: "test".to_string(),
    });
    let atomic1 = AtomicRef::new(data);
    let atomic2 = atomic1.clone();

    assert_eq!(atomic1.get().value, 42);
    assert_eq!(atomic2.get().value, 42);

    // Update atomic1
    atomic1.set(Arc::new(TestData {
        value: 100,
        name: "updated".to_string(),
    }));

    // atomic2 should still have the old value
    assert_eq!(atomic1.get().value, 100);
    assert_eq!(atomic2.get().value, 42);
}

#[test]
fn test_trait_atomic() {
    fn test_atomic<T: Atomic<Value = Arc<i32>>>(atomic: &T) {
        atomic.set(Arc::new(42));
        assert_eq!(*atomic.get(), 42);
        let old = atomic.swap(Arc::new(100));
        assert_eq!(*old, 42);
    }

    let atomic = AtomicRef::new(Arc::new(0));
    test_atomic(&atomic);
}

#[test]
fn test_trait_updatable_atomic() {
    fn test_updatable<T: UpdatableAtomic<Value = Arc<i32>>>(atomic: &T) {
        let new = atomic.update_and_get(|x| Arc::new(*x + 10));
        assert_eq!(*new, 10);
    }

    let atomic = AtomicRef::new(Arc::new(0));
    test_updatable(&atomic);
}

#[test]
fn test_debug_display() {
    let data = Arc::new(42);
    let atomic = AtomicRef::new(data);
    let debug_str = format!("{:?}", atomic);
    assert!(debug_str.contains("42"));
    let display_str = format!("{}", atomic);
    assert_eq!(display_str, "42");
}

#[test]
fn test_arc_reference_counting() {
    let data = Arc::new(TestData {
        value: 42,
        name: "test".to_string(),
    });

    // Initial ref count: 1
    assert_eq!(Arc::strong_count(&data), 1);

    let atomic = AtomicRef::new(data.clone());
    // Ref count: 2 (original + atomic)
    assert_eq!(Arc::strong_count(&data), 2);

    let retrieved = atomic.get();
    // Ref count: 3 (original + atomic + retrieved)
    assert_eq!(Arc::strong_count(&data), 3);

    drop(retrieved);
    // Ref count: 2 (original + atomic)
    assert_eq!(Arc::strong_count(&data), 2);

    drop(atomic);
    // Ref count: 1 (original only)
    assert_eq!(Arc::strong_count(&data), 1);
}
