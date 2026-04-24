/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`qubit_concurrent::task::service::ThreadPool`].

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use qubit_concurrent::task::{
    TaskExecutionError,
    service::{ExecutorService, RejectedExecution, ThreadPool, ThreadPoolBuildError},
};

use super::{create_runtime, create_single_worker_pool, wait_started};

fn ok_unit_task() -> Result<(), io::Error> {
    Ok(())
}

fn ok_usize_task() -> Result<usize, io::Error> {
    Ok(42)
}

#[test]
fn test_thread_pool_submit_acceptance_is_not_task_success() {
    let pool = ThreadPool::new(2).expect("thread pool should be created");

    pool.submit(ok_unit_task as fn() -> Result<(), io::Error>)
        .expect("thread pool should accept shared runnable")
        .get()
        .expect("shared runnable should complete successfully");

    let handle = pool
        .submit(|| Err::<(), _>(io::Error::other("task failed")))
        .expect("thread pool should accept runnable");

    let err = handle
        .get()
        .expect_err("accepted runnable should report task failure through handle");
    assert!(matches!(err, TaskExecutionError::Failed(_)));
    pool.shutdown();
    create_runtime().block_on(pool.await_termination());
}

#[test]
fn test_thread_pool_submit_callable_returns_value() {
    let pool = ThreadPool::new(2).expect("thread pool should be created");

    let handle = pool
        .submit_callable(ok_usize_task as fn() -> Result<usize, io::Error>)
        .expect("thread pool should accept callable");

    assert_eq!(
        handle.get().expect("callable should complete successfully"),
        42,
    );
    pool.shutdown();
    create_runtime().block_on(pool.await_termination());
}

#[tokio::test]
async fn test_thread_pool_handle_can_be_awaited() {
    let pool = ThreadPool::new(2).expect("thread pool should be created");

    let handle = pool
        .submit_callable(ok_usize_task as fn() -> Result<usize, io::Error>)
        .expect("thread pool should accept callable");

    assert_eq!(handle.await.expect("handle should await result"), 42);
    pool.shutdown();
    pool.await_termination().await;
}

#[test]
fn test_thread_pool_shutdown_rejects_new_tasks() {
    let pool = ThreadPool::new(1).expect("thread pool should be created");

    pool.shutdown();
    let result = pool.submit(ok_unit_task as fn() -> Result<(), io::Error>);

    assert!(matches!(result, Err(RejectedExecution::Shutdown)));
    create_runtime().block_on(pool.await_termination());
    assert!(pool.is_shutdown());
    assert!(pool.is_terminated());
}

#[test]
fn test_thread_pool_shutdown_drains_queued_tasks() {
    let pool = create_single_worker_pool();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let first = pool
        .submit(move || {
            started_tx
                .send(())
                .expect("test should receive task start signal");
            release_rx
                .recv()
                .map_err(|err| io::Error::other(err.to_string()))?;
            Ok::<(), io::Error>(())
        })
        .expect("first task should be accepted");
    wait_started(started_rx);
    let second = pool
        .submit_callable(ok_usize_task as fn() -> Result<usize, io::Error>)
        .expect("queued task should be accepted");

    pool.shutdown();
    let rejected = pool.submit(ok_unit_task as fn() -> Result<(), io::Error>);
    release_tx
        .send(())
        .expect("blocking task should receive release signal");
    first
        .get()
        .expect("first task should complete successfully");

    assert!(matches!(rejected, Err(RejectedExecution::Shutdown)));
    assert_eq!(second.get().expect("queued task should still run"), 42);
    create_runtime().block_on(pool.await_termination());
    assert!(pool.is_terminated());
}

#[test]
fn test_thread_pool_shutdown_now_cancels_queued_tasks() {
    let pool = create_single_worker_pool();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let first = pool
        .submit(move || {
            started_tx
                .send(())
                .expect("test should receive task start signal");
            release_rx
                .recv()
                .map_err(|err| io::Error::other(err.to_string()))?;
            Ok::<(), io::Error>(())
        })
        .expect("first task should be accepted");
    wait_started(started_rx);
    let queued = pool
        .submit_callable(ok_usize_task as fn() -> Result<usize, io::Error>)
        .expect("queued task should be accepted");

    let report = pool.shutdown_now();

    assert_eq!(report.queued, 1);
    assert_eq!(report.running, 1);
    assert_eq!(report.cancelled, 1);
    assert!(matches!(queued.get(), Err(TaskExecutionError::Cancelled),));
    release_tx
        .send(())
        .expect("blocking task should receive release signal");
    first.get().expect("running task should complete normally");
    create_runtime().block_on(pool.await_termination());
    assert!(pool.is_terminated());
}

#[test]
fn test_thread_pool_cancel_before_start_reports_cancelled() {
    let pool = create_single_worker_pool();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let first = pool
        .submit(move || {
            started_tx
                .send(())
                .expect("test should receive task start signal");
            release_rx
                .recv()
                .map_err(|err| io::Error::other(err.to_string()))?;
            Ok::<(), io::Error>(())
        })
        .expect("first task should be accepted");
    wait_started(started_rx);
    let queued = pool
        .submit_callable(ok_usize_task as fn() -> Result<usize, io::Error>)
        .expect("queued task should be accepted");

    assert!(queued.cancel());
    assert!(queued.is_done());
    assert!(matches!(queued.get(), Err(TaskExecutionError::Cancelled),));
    pool.shutdown();
    release_tx
        .send(())
        .expect("blocking task should receive release signal");
    first.get().expect("running task should complete normally");
    create_runtime().block_on(pool.await_termination());
}

#[test]
fn test_thread_pool_accessors_and_dynamic_settings() {
    let pool = ThreadPool::builder()
        .core_pool_size(0)
        .maximum_pool_size(2)
        .queue_capacity(1)
        .build()
        .expect("thread pool should be created");

    assert_eq!(pool.queued_count(), 0);
    assert_eq!(pool.running_count(), 0);
    assert_eq!(pool.live_worker_count(), 0);
    assert_eq!(pool.core_pool_size(), 0);
    assert_eq!(pool.maximum_pool_size(), 2);
    assert!(pool.set_core_pool_size(1).is_ok());
    assert!(pool.set_maximum_pool_size(3).is_ok());
    assert!(pool.set_keep_alive(Duration::from_millis(25)).is_ok());
    pool.allow_core_thread_timeout(true);
    assert_eq!(pool.core_pool_size(), 1);
    assert_eq!(pool.maximum_pool_size(), 3);
    assert!(matches!(
        pool.set_core_pool_size(4),
        Err(ThreadPoolBuildError::CorePoolSizeExceedsMaximum { .. }),
    ));
    assert!(matches!(
        pool.set_maximum_pool_size(0),
        Err(ThreadPoolBuildError::ZeroMaximumPoolSize),
    ));
    assert!(pool.set_core_pool_size(2).is_ok());
    assert!(matches!(
        pool.set_maximum_pool_size(1),
        Err(ThreadPoolBuildError::CorePoolSizeExceedsMaximum { .. }),
    ));
    assert!(matches!(
        pool.set_keep_alive(Duration::ZERO),
        Err(ThreadPoolBuildError::ZeroKeepAlive),
    ));
    pool.shutdown();
    create_runtime().block_on(pool.await_termination());
}

#[test]
fn test_thread_pool_reports_worker_spawn_failure() {
    let pool = ThreadPool::builder()
        .pool_size(1)
        .stack_size(usize::MAX)
        .build()
        .expect("thread pool should be created lazily");

    let result = pool.submit(ok_unit_task as fn() -> Result<(), io::Error>);

    assert!(matches!(
        result,
        Err(RejectedExecution::WorkerSpawnFailed { .. }),
    ));
    pool.shutdown();
    create_runtime().block_on(pool.await_termination());
}

#[test]
fn test_thread_pool_cancels_queued_job_when_initial_worker_spawn_fails() {
    let pool = ThreadPool::builder()
        .core_pool_size(0)
        .maximum_pool_size(1)
        .queue_capacity(1)
        .stack_size(usize::MAX)
        .build()
        .expect("thread pool should be created lazily");

    let result = pool.submit(ok_unit_task as fn() -> Result<(), io::Error>);

    assert!(matches!(
        result,
        Err(RejectedExecution::WorkerSpawnFailed { .. }),
    ));
    assert_eq!(pool.queued_count(), 0);
    pool.shutdown();
    create_runtime().block_on(pool.await_termination());
}

#[test]
fn test_rejected_execution_compares_by_variant() {
    let left = RejectedExecution::WorkerSpawnFailed {
        source: Arc::new(io::Error::other("left")),
    };
    let right = RejectedExecution::WorkerSpawnFailed {
        source: Arc::new(io::Error::other("right")),
    };

    assert_eq!(left, right);
    assert_ne!(RejectedExecution::Shutdown, RejectedExecution::Saturated);
    assert_eq!(
        RejectedExecution::Saturated.to_string(),
        "task rejected because the executor service is saturated",
    );
}

#[test]
fn test_thread_pool_shutdown_now_after_shutdown_is_idempotent() {
    let pool = ThreadPool::new(1).expect("thread pool should be created");

    pool.shutdown();
    let report = pool.shutdown_now();

    assert_eq!(report.queued, 0);
    assert_eq!(report.running, 0);
    assert_eq!(report.cancelled, 0);
    create_runtime().block_on(pool.await_termination());
    assert!(pool.is_terminated());
}

/// Runs one stress round where producer threads race submit calls against a
/// concurrent shutdown request.
///
/// # Parameters
///
/// * `use_shutdown_now` - `true` to call `shutdown_now`, otherwise `shutdown`.
///
/// # Returns
///
/// Number of rejected submissions observed by producers.
fn run_submit_shutdown_race_round(use_shutdown_now: bool) -> usize {
    let pool = Arc::new(
        ThreadPool::builder()
            .core_pool_size(0)
            .maximum_pool_size(4)
            .queue_capacity(64)
            .build()
            .expect("thread pool should be created"),
    );
    let producer_count = 6usize;
    let stop = Arc::new(AtomicBool::new(false));
    let start_barrier = Arc::new(std::sync::Barrier::new(producer_count + 1));
    let mut producers = Vec::with_capacity(producer_count);
    for _ in 0..producer_count {
        let pool = Arc::clone(&pool);
        let stop = Arc::clone(&stop);
        let start_barrier = Arc::clone(&start_barrier);
        producers.push(thread::spawn(move || {
            let mut rejected = 0usize;
            start_barrier.wait();
            while !stop.load(Ordering::Acquire) {
                match pool.submit(ok_unit_task as fn() -> Result<(), io::Error>) {
                    Ok(_) => {}
                    Err(RejectedExecution::Shutdown) => {
                        rejected += 1;
                        break;
                    }
                    Err(RejectedExecution::Saturated) => {
                        // Keep producer pressure but yield briefly to avoid
                        // monopolizing CPU in tight saturation loops.
                        thread::yield_now();
                    }
                    Err(RejectedExecution::WorkerSpawnFailed { source }) => {
                        panic!("worker spawn should not fail in race test: {source}");
                    }
                }
            }
            rejected
        }));
    }
    start_barrier.wait();
    thread::sleep(Duration::from_millis(5));

    // Run shutdown in a dedicated thread and bound waiting time using channel
    // timeout so this test fails fast if inflight-drain signaling regresses.
    let (shutdown_done_tx, shutdown_done_rx) = mpsc::channel();
    let pool_for_shutdown = Arc::clone(&pool);
    let shutdown_thread = thread::spawn(move || {
        if use_shutdown_now {
            let _ = pool_for_shutdown.shutdown_now();
        } else {
            pool_for_shutdown.shutdown();
        }
        shutdown_done_tx
            .send(())
            .expect("shutdown completion should be observable");
    });
    shutdown_done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("shutdown should finish under submit race");
    shutdown_thread
        .join()
        .expect("shutdown thread should not panic");

    stop.store(true, Ordering::Release);
    let mut rejected_total = 0usize;
    for producer in producers {
        rejected_total += producer.join().expect("producer thread should not panic");
    }

    let (terminated_tx, terminated_rx) = mpsc::channel();
    let pool_for_wait = Arc::clone(&pool);
    let wait_thread = thread::spawn(move || {
        create_runtime().block_on(pool_for_wait.await_termination());
        terminated_tx
            .send(())
            .expect("termination completion should be observable");
    });
    terminated_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("await_termination should complete under submit race");
    wait_thread.join().expect("wait thread should not panic");

    assert!(pool.is_shutdown());
    assert!(pool.is_terminated());
    rejected_total
}

#[test]
fn test_thread_pool_shutdown_completes_under_submit_race() {
    let mut rejected_rounds = 0usize;
    for _ in 0..40 {
        if run_submit_shutdown_race_round(false) > 0 {
            rejected_rounds += 1;
        }
    }
    assert!(
        rejected_rounds > 0,
        "at least one round should observe submit rejection after shutdown",
    );
}

#[test]
fn test_thread_pool_shutdown_now_completes_under_submit_race() {
    let mut rejected_rounds = 0usize;
    for _ in 0..40 {
        if run_submit_shutdown_race_round(true) > 0 {
            rejected_rounds += 1;
        }
    }
    assert!(
        rejected_rounds > 0,
        "at least one round should observe submit rejection after shutdown_now",
    );
}
