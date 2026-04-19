/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`TokioExecutorService`](qubit_concurrent::TokioExecutorService).

use std::{
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            AtomicUsize,
            Ordering,
        },
    },
    time::Duration,
};

use qubit_concurrent::{
    AsyncExecutor,
    AsyncExecutorService,
    TokioExecutorService,
};

#[tokio::test]
async fn test_tokio_executor_service_shutdown_blocks_new_tasks() {
    let executor = TokioExecutorService::new();
    let ran = Arc::new(AtomicBool::new(false));
    let ran_for_task = Arc::clone(&ran);
    executor.shutdown();

    executor.spawn(async move {
        ran_for_task.store(true, Ordering::Release);
    });

    tokio::task::yield_now().await;
    assert!(!ran.load(Ordering::Acquire));
    assert!(executor.is_shutdown());
    assert!(executor.is_terminated());
}

#[tokio::test]
async fn test_tokio_executor_service_shutdown_now_aborts_running_tasks() {
    let executor = TokioExecutorService::new();
    let finished = Arc::new(AtomicBool::new(false));
    let finished_for_task = Arc::clone(&finished);

    executor.spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        finished_for_task.store(true, Ordering::Release);
    });

    tokio::task::yield_now().await;
    executor.shutdown_now();
    executor.await_termination().await;

    assert!(executor.is_shutdown());
    assert!(executor.is_terminated());
    assert!(!finished.load(Ordering::Acquire));
}

#[tokio::test]
async fn test_tokio_executor_service_await_termination_waits_for_tasks() {
    let executor = TokioExecutorService::new();
    let completed = Arc::new(AtomicUsize::new(0));

    for _ in 0..3 {
        let completed_for_task = Arc::clone(&completed);
        executor.spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            completed_for_task.fetch_add(1, Ordering::AcqRel);
        });
    }

    executor.shutdown();
    executor.await_termination().await;

    assert_eq!(completed.load(Ordering::Acquire), 3);
    assert!(executor.is_shutdown());
    assert!(executor.is_terminated());
}
