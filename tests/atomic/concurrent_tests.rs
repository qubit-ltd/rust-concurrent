/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

use prism3_concurrent::atomic::{
    AtomicBool, AtomicF32, AtomicI32, AtomicI64, AtomicRef, AtomicU32, AtomicUsize,
};
use std::sync::atomic::{AtomicUsize as StdAtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

const NUM_THREADS: usize = 10;
const ITERATIONS_PER_THREAD: usize = 1000;

// Test concurrent increments
#[test]
fn test_concurrent_increment() {
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..ITERATIONS_PER_THREAD {
                counter.increment_and_get();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.get(), (NUM_THREADS * ITERATIONS_PER_THREAD) as i32);
}

// Test concurrent decrements
#[test]
fn test_concurrent_decrement() {
    let counter = Arc::new(AtomicI64::new(10000));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                counter.decrement_and_get();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.get(), 10000 - (NUM_THREADS * 100) as i64);
}

// Test concurrent CAS operations
#[test]
fn test_concurrent_cas() {
    let atomic = Arc::new(AtomicU32::new(0));
    let success_count = Arc::new(StdAtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let atomic = atomic.clone();
        let success_count = success_count.clone();
        let handle = thread::spawn(move || {
            let mut current = atomic.get();
            loop {
                match atomic.compare_and_set_weak(current, current + 1) {
                    Ok(_) => {
                        success_count.fetch_add(1, Ordering::Relaxed);
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

    assert_eq!(atomic.get(), NUM_THREADS as u32);
    assert_eq!(success_count.load(Ordering::Relaxed), NUM_THREADS);
}

// Test concurrent swap operations
#[test]
fn test_concurrent_swap() {
    let atomic = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];
    let sum = Arc::new(StdAtomicUsize::new(0));

    for i in 0..NUM_THREADS {
        let atomic = atomic.clone();
        let sum = sum.clone();
        let handle = thread::spawn(move || {
            let old = atomic.swap((i + 1) as i32);
            sum.fetch_add(old as usize, Ordering::Relaxed);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // The final value should be one of the thread IDs
    let final_value = atomic.get();
    assert!(final_value >= 1 && final_value <= NUM_THREADS as i32);
}

// Test concurrent boolean flag operations
#[test]
fn test_concurrent_flag() {
    let flag = Arc::new(AtomicBool::new(false));
    let success_count = Arc::new(StdAtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let flag = flag.clone();
        let success_count = success_count.clone();
        let handle = thread::spawn(move || {
            if flag.compare_and_set_if_false(true).is_ok() {
                success_count.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Only one thread should succeed
    assert_eq!(flag.get(), true);
    assert_eq!(success_count.load(Ordering::Relaxed), 1);
}

// Test concurrent toggle operations
#[test]
fn test_concurrent_toggle() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
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

    // After even number of toggles, should be false
    assert_eq!(flag.get(), false);
}

// Test concurrent floating-point additions
#[test]
fn test_concurrent_float_add() {
    let sum = Arc::new(AtomicF32::new(0.0));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
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

    // Due to floating point precision, result may not be exact
    let result = sum.get();
    let expected = (NUM_THREADS * 100) as f32 * 0.01;
    assert!((result - expected).abs() < 0.1);
}

// Test concurrent reference updates
#[test]
fn test_concurrent_ref_update() {
    let atomic = Arc::new(AtomicRef::new(Arc::new(0)));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                atomic.update_and_get(|current| {
                    let value = **current;
                    Arc::new(value + 1)
                });
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*atomic.get(), NUM_THREADS * 100);
}

// Test concurrent accumulate operations
#[test]
fn test_concurrent_accumulate() {
    let atomic = Arc::new(AtomicI32::new(1));
    let mut handles = vec![];

    for _ in 0..5 {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            atomic.accumulate_and_get(2, |a, b| a * b);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 1 * 2^5 = 32
    assert_eq!(atomic.get(), 32);
}

// Test concurrent max operations
#[test]
fn test_concurrent_max() {
    let atomic = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for i in 0..NUM_THREADS {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            atomic.max_and_get((i * 10) as i32);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(atomic.get(), ((NUM_THREADS - 1) * 10) as i32);
}

// Test concurrent min operations
#[test]
fn test_concurrent_min() {
    let atomic = Arc::new(AtomicI32::new(1000));
    let mut handles = vec![];

    for i in 0..NUM_THREADS {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            atomic.min_and_get((100 - i * 5) as i32);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(atomic.get(), (100 - (NUM_THREADS - 1) * 5) as i32);
}

// Test barrier synchronization with atomic operations
#[test]
fn test_barrier_sync() {
    let counter = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let counter = counter.clone();
        let barrier = barrier.clone();
        let handle = thread::spawn(move || {
            // All threads wait at the barrier
            barrier.wait();
            // Then all increment simultaneously
            counter.increment_and_get();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.get(), NUM_THREADS);
}

// Test producer-consumer pattern with atomic flag
#[test]
fn test_producer_consumer() {
    let data = Arc::new(AtomicI32::new(0));
    let ready = Arc::new(AtomicBool::new(false));

    let data_clone = data.clone();
    let ready_clone = ready.clone();

    // Producer thread
    let producer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        data_clone.set(42);
        ready_clone.set(true);
    });

    // Consumer thread
    let consumer = thread::spawn(move || {
        while !ready.get() {
            thread::yield_now();
        }
        data.get()
    });

    producer.join().unwrap();
    let result = consumer.join().unwrap();
    assert_eq!(result, 42);
}

// Test spinlock-like pattern
#[test]
fn test_spinlock_pattern() {
    let lock = Arc::new(AtomicBool::new(false));
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for _ in 0..NUM_THREADS {
        let lock = lock.clone();
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                // Acquire lock
                while lock.compare_and_set_if_false(true).is_err() {
                    thread::yield_now();
                }

                // Critical section
                let value = counter.get();
                thread::yield_now(); // Simulate some work
                counter.set(value + 1);

                // Release lock
                lock.set(false);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.get(), (NUM_THREADS * 10) as i32);
}

// Test concurrent bitwise operations
#[test]
fn test_concurrent_bitwise() {
    let atomic = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];

    for i in 0..NUM_THREADS {
        let atomic = atomic.clone();
        let handle = thread::spawn(move || {
            let bit = 1u32 << (i % 32);
            atomic.get_and_bitor(bit);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = atomic.get();
    // Check that all bits were set
    for i in 0..NUM_THREADS.min(32) {
        let bit = 1u32 << i;
        assert_eq!(result & bit, bit);
    }
}

// Test memory ordering visibility
#[test]
fn test_memory_ordering_visibility() {
    let data = Arc::new(AtomicI32::new(0));
    let flag = Arc::new(AtomicBool::new(false));

    let data_clone = data.clone();
    let flag_clone = flag.clone();

    let writer = thread::spawn(move || {
        data_clone.set(42);
        flag_clone.set(true);
    });

    let reader = thread::spawn(move || {
        while !flag.get() {
            thread::yield_now();
        }
        data.get()
    });

    writer.join().unwrap();
    let result = reader.join().unwrap();
    assert_eq!(result, 42);
}
