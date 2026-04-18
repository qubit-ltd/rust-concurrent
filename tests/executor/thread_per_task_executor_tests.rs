/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`ThreadPerTaskExecutor`](qubit_concurrent::ThreadPerTaskExecutor).

use std::{
    sync::mpsc,
    time::Duration,
};

use qubit_concurrent::{Executor, ThreadPerTaskExecutor};

#[test]
fn test_thread_per_task_executor_execute_runs_task() {
    let executor = ThreadPerTaskExecutor;
    let (sender, receiver) = mpsc::channel();

    executor.execute(Box::new(move || {
        sender
            .send(42usize)
            .expect("Failed to send task result to test thread");
    }));

    let received = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("Task did not complete within timeout");
    assert_eq!(received, 42);
}
