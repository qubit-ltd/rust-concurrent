/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`TaskHandle`](qubit_concurrent::task::TaskHandle).

use std::{
    io,
    thread,
    time::Duration,
};

use qubit_concurrent::task::executor::{
    Executor,
    ThreadPerTaskExecutor,
};

#[tokio::test]
async fn test_task_handle_await_returns_value() {
    let executor = ThreadPerTaskExecutor;

    let handle = executor.call(|| Ok::<usize, io::Error>(42));

    assert_eq!(
        handle.await.expect("task handle should await task result"),
        42,
    );
}

#[test]
fn test_task_handle_is_done_reports_completion() {
    let executor = ThreadPerTaskExecutor;

    let handle = executor.call(|| Ok::<usize, io::Error>(42));
    for _ in 0..100 {
        if handle.is_done() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(handle.is_done());
    assert_eq!(handle.get().expect("task should complete successfully"), 42);
}
