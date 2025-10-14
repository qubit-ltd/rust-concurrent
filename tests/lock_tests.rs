/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Concurrent Lock Module Tests

use prism3_concurrent::{ArcAsyncMutex, ArcAsyncRwLock, ArcMutex, ArcRwLock};
use std::sync::Arc;
use std::thread;

#[test]
fn test_arc_mutex() {
    let mutex = ArcMutex::new(0);

    // Test basic operations
    let result = mutex.with_lock(|value| {
        *value += 1;
        *value
    });
    assert_eq!(result, 1);

    // Test trying to acquire lock
    let result = mutex.try_with_lock(|value| *value).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_arc_mutex_clone() {
    let mutex = ArcMutex::new(0);
    let mutex_clone = mutex.clone();

    // Test that cloned lock shares data
    let result = mutex_clone.with_lock(|value| {
        *value += 1;
        *value
    });
    assert_eq!(result, 1);

    // Verify that original lock can see changes
    let result = mutex.with_lock(|value| *value);
    assert_eq!(result, 1);
}

#[test]
fn test_arc_rw_lock() {
    let rw_lock = ArcRwLock::new(0);

    // Test read lock
    let result = rw_lock.with_read_lock(|value| *value);
    assert_eq!(result, 0);

    // Test write lock
    let result = rw_lock.with_write_lock(|value| {
        *value += 1;
        *value
    });
    assert_eq!(result, 1);
}

#[test]
fn test_arc_rw_lock_clone() {
    let rw_lock = ArcRwLock::new(0);
    let rw_lock_clone = rw_lock.clone();

    // Test cloned read-write lock
    let result = rw_lock_clone.with_write_lock(|value| {
        *value += 1;
        *value
    });
    assert_eq!(result, 1);

    // Verify that original lock can see changes
    let result = rw_lock.with_read_lock(|value| *value);
    assert_eq!(result, 1);
}

#[tokio::test]
async fn test_arc_async_mutex() {
    let async_mutex = ArcAsyncMutex::new(0);

    // Test async lock
    let result = async_mutex
        .with_lock(|value| {
            *value += 1;
            *value
        })
        .await;
    assert_eq!(result, 1);

    // Test trying to acquire lock
    let result = async_mutex.try_with_lock(|value| *value).unwrap();
    assert_eq!(result, 1);
}

#[tokio::test]
async fn test_arc_async_mutex_clone() {
    let async_mutex = ArcAsyncMutex::new(0);
    let async_mutex_clone = async_mutex.clone();

    // Test cloned async lock
    let result = async_mutex_clone
        .with_lock(|value| {
            *value += 1;
            *value
        })
        .await;
    assert_eq!(result, 1);

    // Verify that original lock can see changes
    let result = async_mutex.with_lock(|value| *value).await;
    assert_eq!(result, 1);
}

#[tokio::test]
async fn test_arc_async_rw_lock() {
    let async_rw_lock = ArcAsyncRwLock::new(0);

    // Test read lock
    let result = async_rw_lock.with_read_lock(|value| *value).await;
    assert_eq!(result, 0);

    // Test write lock
    let result = async_rw_lock
        .with_write_lock(|value| {
            *value += 1;
            *value
        })
        .await;
    assert_eq!(result, 1);
}

#[tokio::test]
async fn test_arc_async_rw_lock_clone() {
    let async_rw_lock = ArcAsyncRwLock::new(0);
    let async_rw_lock_clone = async_rw_lock.clone();

    // Test cloned async read-write lock
    let result = async_rw_lock_clone
        .with_write_lock(|value| {
            *value += 1;
            *value
        })
        .await;
    assert_eq!(result, 1);

    // Verify that original lock can see changes
    let result = async_rw_lock.with_read_lock(|value| *value).await;
    assert_eq!(result, 1);
}

#[test]
fn test_arc_mutex_try_with_lock_returns_none() {
    use std::sync::Barrier;

    let mutex = Arc::new(ArcMutex::new(0));
    let barrier = Arc::new(Barrier::new(2));

    let mutex_clone = mutex.clone();
    let barrier_clone = barrier.clone();

    // Hold the lock in another thread
    let handle = thread::spawn(move || {
        mutex_clone.with_lock(|value| {
            *value += 1;
            // Notify main thread that it can try to acquire the lock
            barrier_clone.wait();
            // Hold the lock for some time
            thread::sleep(std::time::Duration::from_millis(100));
        });
    });

    // Wait for child thread to acquire the lock
    barrier.wait();

    // Try to acquire lock, should return None
    let result = mutex.try_with_lock(|value| *value);
    assert!(
        result.is_none(),
        "Expected None when lock is held by another thread"
    );

    // Wait for child thread to complete
    handle.join().unwrap();

    // Now should be able to successfully acquire the lock
    let result = mutex.try_with_lock(|value| *value);
    assert_eq!(result, Some(1));
}

#[tokio::test]
async fn test_arc_async_mutex_try_with_lock_returns_none() {
    use std::sync::Barrier;

    let async_mutex = Arc::new(ArcAsyncMutex::new(0));
    let barrier = Arc::new(Barrier::new(2));

    let async_mutex_clone = async_mutex.clone();
    let barrier_clone = barrier.clone();

    // Hold the lock in another thread (note: using thread instead of tokio task)
    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            async_mutex_clone
                .with_lock(|value| {
                    *value += 1;
                    // Notify main thread that it can try to acquire the lock
                    barrier_clone.wait();
                    // Hold the lock for some time
                    std::thread::sleep(std::time::Duration::from_millis(100));
                })
                .await;
        });
    });

    // Wait for child thread to acquire the lock
    barrier.wait();

    // Try to acquire lock, should return None
    let result = async_mutex.try_with_lock(|value| *value);
    assert!(
        result.is_none(),
        "Expected None when lock is held by another thread"
    );

    // Wait for child thread to complete
    handle.join().unwrap();

    // Now should be able to successfully acquire the lock
    let result = async_mutex.try_with_lock(|value| *value);
    assert_eq!(result, Some(1));
}

#[test]
fn test_arc_mutex_try_with_lock_poisoned() {
    use std::sync::Barrier;

    let mutex = Arc::new(ArcMutex::new(0));
    let barrier = Arc::new(Barrier::new(2));

    let mutex_clone = mutex.clone();
    let barrier_clone = barrier.clone();

    // Hold the lock and panic in another thread
    let handle = thread::spawn(move || {
        mutex_clone.with_lock(|value| {
            *value += 1;
            // Notify main thread that lock has been acquired
            barrier_clone.wait();
            // Panic while holding the lock, causing the lock to be poisoned
            panic!("intentional panic to poison the lock");
        });
    });

    // Wait for child thread to acquire the lock
    barrier.wait();

    // Wait for child thread to complete panicking (will poison the lock)
    let _ = handle.join();

    // Try to acquire poisoned lock, should return None (because try_lock returns Err)
    // Standard library behavior: for poisoned lock, try_lock returns Err(TryLockError::Poisoned)
    // Current implementation treats this as returning None
    let result = mutex.try_with_lock(|value| *value);

    // Since current implementation treats all Err as None, it will be None here
    // This masks the fact that the lock has been poisoned, but for try_with_lock this is acceptable behavior
    assert!(
        result.is_none(),
        "Expected None for poisoned lock, got {:?}",
        result
    );
}

#[test]
#[should_panic(expected = "PoisonError")]
fn test_arc_mutex_with_lock_poisoned() {
    use std::sync::Barrier;

    let mutex = Arc::new(ArcMutex::new(0));
    let barrier = Arc::new(Barrier::new(2));

    let mutex_clone = mutex.clone();
    let barrier_clone = barrier.clone();

    // Hold the lock and panic in another thread
    let handle = thread::spawn(move || {
        mutex_clone.with_lock(|value| {
            *value += 1;
            // Notify main thread that lock has been acquired
            barrier_clone.wait();
            // Panic while holding the lock, causing the lock to be poisoned
            panic!("intentional panic to poison the lock");
        });
    });

    // Wait for child thread to acquire the lock
    barrier.wait();

    // Wait for child thread to complete panicking (will poison the lock)
    let _ = handle.join();

    // Try to acquire poisoned lock with with_lock (not try_with_lock)
    // This should panic because with_lock uses unwrap() on line 116
    mutex.with_lock(|_| {});
}

#[test]
fn test_concurrent_access() {
    let mutex = ArcMutex::new(0);
    let mutex = Arc::new(mutex);

    let mut handles = vec![];

    // Create multiple threads accessing the lock concurrently
    for _ in 0..10 {
        let mutex = Arc::clone(&mutex);
        let handle = thread::spawn(move || {
            mutex.with_lock(|value| {
                *value += 1;
            });
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final value
    let result = mutex.with_lock(|value| *value);
    assert_eq!(result, 10);
}
