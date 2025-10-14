/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Lock Wrappers
//!
//! Provides wrapper functionality for synchronous and asynchronous locks,
//! for safely accessing shared data in both synchronous and asynchronous environments.
//!
//! # Author
//!
//! Haixing Hu

use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};

/// Synchronous Mutex Wrapper
///
/// Provides an encapsulation of synchronous mutex for protecting shared data
/// in synchronous environments. Supports safe access and modification of shared
/// data across multiple threads.
///
/// # Features
///
/// - Synchronously acquires locks, may block threads
/// - Supports trying to acquire locks (non-blocking)
/// - Thread-safe, supports multi-threaded sharing
/// - Automatic lock management through RAII ensures proper lock release
///
/// # Usage Example
///
/// ```rust,ignore
/// use prism3_concurrent::ArcMutex;
/// use std::sync::Arc;
///
/// let counter = ArcMutex::new(0);
/// let counter = Arc::new(counter);
///
/// // Synchronously modify data
/// counter.with_lock(|c| {
///     *c += 1;
///     println!("Counter: {}", *c);
/// });
///
/// // Try to acquire lock
/// if let Some(value) = counter.try_with_lock(|c| *c) {
///     println!("Current value: {}", value);
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub struct ArcMutex<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> ArcMutex<T> {
    /// Creates a new synchronous mutex lock
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be protected
    ///
    /// # Returns
    ///
    /// Returns a new `ArcMutex` instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcMutex;
    ///
    /// let lock = ArcMutex::new(42);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(data)),
        }
    }

    /// Acquires the lock and executes an operation
    ///
    /// Synchronously acquires the lock, executes the provided closure, and then
    /// automatically releases the lock. This is the recommended usage pattern as it
    /// ensures proper lock release.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the lock
    ///
    /// # Returns
    ///
    /// Returns the result of executing the closure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcMutex;
    ///
    /// let counter = ArcMutex::new(0);
    ///
    /// let result = counter.with_lock(|c| {
    ///     *c += 1;
    ///     *c
    /// });
    ///
    /// println!("Counter value: {}", result);
    /// ```
    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock().unwrap();
        f(&mut *guard)
    }

    /// Attempts to acquire the lock
    ///
    /// Attempts to immediately acquire the lock. If the lock is already held by
    /// another thread, returns `None`. This is a non-blocking operation.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the lock
    ///
    /// # Returns
    ///
    /// * `Some(R)` - If the lock was successfully acquired and the closure executed
    /// * `None` - If the lock is already held by another thread
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcMutex;
    ///
    /// let counter = ArcMutex::new(0);
    ///
    /// // Try to acquire lock
    /// if let Some(value) = counter.try_with_lock(|c| *c) {
    ///     println!("Current value: {}", value);
    /// } else {
    ///     println!("Lock is busy");
    /// }
    /// ```
    pub fn try_with_lock<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        if let Ok(mut guard) = self.inner.try_lock() {
            Some(f(&mut *guard))
        } else {
            None
        }
    }
}

impl<T> Clone for ArcMutex<T> {
    /// Clones the synchronous mutex
    ///
    /// Creates a new `ArcMutex` instance that shares the same underlying lock
    /// with the original instance. This allows multiple threads to hold references
    /// to the same lock simultaneously.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Synchronous Read-Write Lock Wrapper
///
/// Provides an encapsulation of synchronous read-write lock, supporting multiple
/// read operations or a single write operation. Read operations can execute
/// concurrently, while write operations have exclusive access.
///
/// # Features
///
/// - Supports multiple concurrent read operations
/// - Write operations have exclusive access, mutually exclusive with read operations
/// - Synchronously acquires locks, may block threads
/// - Thread-safe, supports multi-threaded sharing
/// - Automatic lock management through RAII ensures proper lock release
///
/// # Use Cases
///
/// Suitable for read-heavy scenarios such as caching, configuration management, etc.
///
/// # Usage Example
///
/// ```rust,ignore
/// use prism3_concurrent::ArcRwLock;
/// use std::sync::Arc;
///
/// let data = ArcRwLock::new(String::from("Hello"));
/// let data = Arc::new(data);
///
/// // Multiple read operations can execute concurrently
/// data.with_read_lock(|s| {
///     println!("Read: {}", s);
/// });
///
/// // Write operations have exclusive access
/// data.with_write_lock(|s| {
///     s.push_str(" World!");
///     println!("Write: {}", s);
/// });
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub struct ArcRwLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> ArcRwLock<T> {
    /// Creates a new synchronous read-write lock
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be protected
    ///
    /// # Returns
    ///
    /// Returns a new `ArcRwLock` instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcRwLock;
    ///
    /// let rw_lock = ArcRwLock::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// Acquires the read lock and executes an operation
    ///
    /// Synchronously acquires the read lock, executes the provided closure, and then
    /// automatically releases the lock. Multiple read operations can execute concurrently.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the read lock, can only read data
    ///
    /// # Returns
    ///
    /// Returns the result of executing the closure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcRwLock;
    ///
    /// let data = ArcRwLock::new(vec![1, 2, 3]);
    ///
    /// let length = data.with_read_lock(|v| v.len());
    /// println!("Vector length: {}", length);
    /// ```
    pub fn with_read_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read().unwrap();
        f(&*guard)
    }

    /// Acquires the write lock and executes an operation
    ///
    /// Synchronously acquires the write lock, executes the provided closure, and then
    /// automatically releases the lock. Write operations have exclusive access, mutually
    /// exclusive with read operations.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the write lock, can modify data
    ///
    /// # Returns
    ///
    /// Returns the result of executing the closure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcRwLock;
    ///
    /// let data = ArcRwLock::new(vec![1, 2, 3]);
    ///
    /// data.with_write_lock(|v| {
    ///     v.push(4);
    ///     println!("Added element, new length: {}", v.len());
    /// });
    /// ```
    pub fn with_write_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write().unwrap();
        f(&mut *guard)
    }
}

impl<T> Clone for ArcRwLock<T> {
    /// Clones the synchronous read-write lock
    ///
    /// Creates a new `ArcRwLock` instance that shares the same underlying lock
    /// with the original instance. This allows multiple threads to hold references
    /// to the same lock simultaneously.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Asynchronous Mutex Wrapper
///
/// Provides an encapsulation of asynchronous mutex for protecting shared data
/// in asynchronous environments. Supports safe access and modification of shared
/// data across multiple asynchronous tasks.
///
/// # Features
///
/// - Asynchronously acquires locks, does not block threads
/// - Supports trying to acquire locks (non-blocking)
/// - Thread-safe, supports multi-threaded sharing
/// - Automatic lock management through RAII ensures proper lock release
///
/// # Usage Example
///
/// ```rust,ignore
/// use prism3_concurrent::ArcAsyncMutex;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let counter = ArcAsyncMutex::new(0);
///     let counter = Arc::new(counter);
///
///     // Asynchronously modify data
///     counter.with_lock(|c| {
///         *c += 1;
///         println!("Counter: {}", *c);
///     }).await;
///
///     // Try to acquire lock
///     if let Some(value) = counter.try_with_lock(|c| *c) {
///         println!("Current value: {}", value);
///     }
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub struct ArcAsyncMutex<T> {
    inner: Arc<AsyncMutex<T>>,
}

impl<T> ArcAsyncMutex<T> {
    /// Creates a new asynchronous mutex lock
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be protected
    ///
    /// # Returns
    ///
    /// Returns a new `ArcAsyncMutex` instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcAsyncMutex;
    ///
    /// let lock = ArcAsyncMutex::new(42);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(data)),
        }
    }

    /// Acquires the lock and executes an operation
    ///
    /// Asynchronously acquires the lock, executes the provided closure, and then
    /// automatically releases the lock. This is the recommended usage pattern as it
    /// ensures proper lock release.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the lock
    ///
    /// # Returns
    ///
    /// Returns the result of executing the closure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcAsyncMutex;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let counter = ArcAsyncMutex::new(0);
    ///
    ///     let result = counter.with_lock(|c| {
    ///         *c += 1;
    ///         *c
    ///     }).await;
    ///
    ///     println!("Counter value: {}", result);
    /// }
    /// ```
    pub async fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock().await;
        f(&mut *guard)
    }

    /// Attempts to acquire the lock
    ///
    /// Attempts to immediately acquire the lock. If the lock is already held by
    /// another task, returns `None`. This is a non-blocking operation.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the lock
    ///
    /// # Returns
    ///
    /// * `Some(R)` - If the lock was successfully acquired and the closure executed
    /// * `None` - If the lock is already held by another task
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcAsyncMutex;
    ///
    /// let counter = ArcAsyncMutex::new(0);
    ///
    /// // Try to acquire lock
    /// if let Some(value) = counter.try_with_lock(|c| *c) {
    ///     println!("Current value: {}", value);
    /// } else {
    ///     println!("Lock is busy");
    /// }
    /// ```
    pub fn try_with_lock<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        if let Ok(mut guard) = self.inner.try_lock() {
            Some(f(&mut *guard))
        } else {
            None
        }
    }
}

impl<T> Clone for ArcAsyncMutex<T> {
    /// Clones the asynchronous mutex
    ///
    /// Creates a new `ArcAsyncMutex` instance that shares the same underlying lock
    /// with the original instance. This allows multiple tasks to hold references
    /// to the same lock simultaneously.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Asynchronous Read-Write Lock Wrapper
///
/// Provides an encapsulation of asynchronous read-write lock, supporting multiple
/// read operations or a single write operation. Read operations can execute
/// concurrently, while write operations have exclusive access.
///
/// # Features
///
/// - Supports multiple concurrent read operations
/// - Write operations have exclusive access, mutually exclusive with read operations
/// - Asynchronously acquires locks, does not block threads
/// - Thread-safe, supports multi-threaded sharing
/// - Automatic lock management through RAII ensures proper lock release
///
/// # Use Cases
///
/// Suitable for read-heavy scenarios such as caching, configuration management, etc.
///
/// # Usage Example
///
/// ```rust,ignore
/// use prism3_concurrent::ArcAsyncRwLock;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let data = ArcAsyncRwLock::new(String::from("Hello"));
///     let data = Arc::new(data);
///
///     // Multiple read operations can execute concurrently
///     data.with_read_lock(|s| {
///         println!("Read: {}", s);
///     }).await;
///
///     // Write operations have exclusive access
///     data.with_write_lock(|s| {
///         s.push_str(" World!");
///         println!("Write: {}", s);
///     }).await;
/// }
/// ```
///
/// # Author
///
/// Haixing Hu
///
pub struct ArcAsyncRwLock<T> {
    inner: Arc<AsyncRwLock<T>>,
}

impl<T> ArcAsyncRwLock<T> {
    /// Creates a new asynchronous read-write lock
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be protected
    ///
    /// # Returns
    ///
    /// Returns a new `ArcAsyncRwLock` instance
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcAsyncRwLock;
    ///
    /// let rw_lock = ArcAsyncRwLock::new(vec![1, 2, 3]);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(AsyncRwLock::new(data)),
        }
    }

    /// Acquires the read lock and executes an operation
    ///
    /// Asynchronously acquires the read lock, executes the provided closure, and then
    /// automatically releases the lock. Multiple read operations can execute concurrently.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the read lock, can only read data
    ///
    /// # Returns
    ///
    /// Returns the result of executing the closure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcAsyncRwLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let data = ArcAsyncRwLock::new(vec![1, 2, 3]);
    ///
    ///     let length = data.with_read_lock(|v| v.len()).await;
    ///     println!("Vector length: {}", length);
    /// }
    /// ```
    pub async fn with_read_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read().await;
        f(&*guard)
    }

    /// Acquires the write lock and executes an operation
    ///
    /// Asynchronously acquires the write lock, executes the provided closure, and then
    /// automatically releases the lock. Write operations have exclusive access, mutually
    /// exclusive with read operations.
    ///
    /// # Arguments
    ///
    /// * `f` - The closure to be executed while holding the write lock, can modify data
    ///
    /// # Returns
    ///
    /// Returns the result of executing the closure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use prism3_concurrent::ArcAsyncRwLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let data = ArcAsyncRwLock::new(vec![1, 2, 3]);
    ///
    ///     data.with_write_lock(|v| {
    ///         v.push(4);
    ///         println!("Added element, new length: {}", v.len());
    ///     }).await;
    /// }
    /// ```
    pub async fn with_write_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write().await;
        f(&mut *guard)
    }
}

impl<T> Clone for ArcAsyncRwLock<T> {
    /// Clones the asynchronous read-write lock
    ///
    /// Creates a new `ArcAsyncRwLock` instance that shares the same underlying lock
    /// with the original instance. This allows multiple tasks to hold references
    /// to the same lock simultaneously.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
