/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`ThreadPerTaskExecutorService`](qubit_concurrent::ThreadPerTaskExecutorService).

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};

use qubit_concurrent::{Executor, ExecutorService, ThreadPerTaskExecutorService};

/// Creates a current-thread Tokio runtime for driving async termination APIs in sync tests.
fn create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime for executor tests")
}

#[test]
fn test_thread_per_task_executor_service_shutdown_blocks_new_tasks() {
    let executor = ThreadPerTaskExecutorService::new();
    let (sender, receiver) = mpsc::channel();
    executor.shutdown();

    executor.execute(Box::new(move || {
        let _ = sender.send(1usize);
    }));

    assert!(
        receiver.recv_timeout(Duration::from_millis(200)).is_err(),
        "Task should not run after shutdown"
    );
    assert!(executor.is_shutdown());
}

#[test]
fn test_thread_per_task_executor_service_await_termination_waits_for_tasks() {
    let executor = ThreadPerTaskExecutorService::new();
    let completed = Arc::new(AtomicBool::new(false));
    let completed_for_task = Arc::clone(&completed);

    executor.execute(Box::new(move || {
        std::thread::sleep(Duration::from_millis(80));
        completed_for_task.store(true, Ordering::Release);
    }));

    executor.shutdown();
    create_runtime().block_on(executor.await_termination());

    assert!(executor.is_terminated());
    assert!(completed.load(Ordering::Acquire));
}

#[test]
fn test_thread_per_task_executor_service_shutdown_now_sets_shutdown_and_returns_empty() {
    let executor = ThreadPerTaskExecutorService::new();
    let queued = executor.shutdown_now();

    assert!(queued.is_empty(), "shutdown_now should return no queued tasks");
    assert!(executor.is_shutdown(), "shutdown_now should set shutdown flag");
}

#[test]
fn test_thread_per_task_executor_service_is_terminated_false_before_shutdown() {
    let executor = ThreadPerTaskExecutorService::new();

    assert!(
        !executor.is_terminated(),
        "Executor should not be terminated before shutdown"
    );
}
