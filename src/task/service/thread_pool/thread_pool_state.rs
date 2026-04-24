/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
use std::time::Duration;

use super::thread_pool_config::ThreadPoolConfig;
use super::thread_pool_lifecycle::ThreadPoolLifecycle;

/// Mutable pool state protected by
/// [`super::thread_pool_inner::ThreadPoolInner::state_monitor`].
pub(super) struct ThreadPoolState {
    /// Current lifecycle state controlling submissions and worker exits.
    pub(super) lifecycle: ThreadPoolLifecycle,
    /// Optional maximum number of queued jobs.
    pub(super) queue_capacity: Option<usize>,
    /// Number of worker loops that have not exited.
    pub(super) live_workers: usize,
    /// Number of live workers currently waiting for work.
    pub(super) idle_workers: usize,
    /// Current configured core pool size.
    pub(super) core_pool_size: usize,
    /// Current configured maximum pool size.
    pub(super) maximum_pool_size: usize,
    /// Current idle timeout for workers allowed to retire.
    pub(super) keep_alive: Duration,
    /// Whether core workers are allowed to time out while idle.
    pub(super) allow_core_thread_timeout: bool,
    /// Index assigned to the next spawned worker.
    pub(super) next_worker_index: usize,
}

impl ThreadPoolState {
    /// Builds the initial mutex-protected pool state for a newly created pool.
    ///
    /// Lifecycle starts as [`ThreadPoolLifecycle::Running`], and sizing or
    /// policy fields are copied from `config`.
    ///
    /// # Parameters
    ///
    /// * `config` - Full [`ThreadPoolConfig`]; this constructor reads
    ///   `queue_capacity`, `core_pool_size`, `maximum_pool_size`, `keep_alive`,
    ///   and `allow_core_thread_timeout`. It does not read `thread_name_prefix`
    ///   or `stack_size`.
    ///
    /// # Returns
    ///
    /// A [`ThreadPoolState`] ready to be wrapped by
    /// [`ThreadPoolInner::state_monitor`](super::thread_pool_inner::ThreadPoolInner::state_monitor).
    ///
    /// # Note
    ///
    /// [`ThreadPoolInner::new`](super::thread_pool_inner::ThreadPoolInner::new)
    /// takes ownership of `config` for this call but must keep the thread name
    /// prefix and stack size for spawning workers; it typically
    /// [`std::mem::take`]s `thread_name_prefix` and copies `stack_size` before
    /// passing the remaining `config` here, so the prefix field in the moved
    /// value may be empty and is ignored.
    pub(super) fn new(config: ThreadPoolConfig) -> Self {
        Self {
            lifecycle: ThreadPoolLifecycle::Running,
            queue_capacity: config.queue_capacity,
            live_workers: 0,
            idle_workers: 0,
            core_pool_size: config.core_pool_size,
            maximum_pool_size: config.maximum_pool_size,
            keep_alive: config.keep_alive,
            allow_core_thread_timeout: config.allow_core_thread_timeout,
            next_worker_index: 0,
        }
    }

    /// Returns whether an idle worker should use a timed wait.
    ///
    /// # Returns
    ///
    /// `true` when core timeout is enabled or the live worker count exceeds
    /// the core pool size.
    pub(super) fn worker_wait_is_timed(&self) -> bool {
        self.allow_core_thread_timeout || self.live_workers > self.core_pool_size
    }

    /// Returns whether an idle worker may retire now.
    ///
    /// # Returns
    ///
    /// `true` when the worker count exceeds the maximum size, or when timeout
    /// policy allows an idle worker to exit.
    pub(super) fn idle_worker_can_retire(&self) -> bool {
        self.live_workers > self.maximum_pool_size
            || (self.worker_wait_is_timed()
                && (self.live_workers > self.core_pool_size || self.allow_core_thread_timeout))
    }
}
