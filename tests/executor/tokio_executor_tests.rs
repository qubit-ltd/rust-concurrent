/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`TokioExecutor`](qubit_concurrent::TokioExecutor).

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use qubit_concurrent::{AsyncExecutor, TokioExecutor};

#[tokio::test]
async fn test_tokio_executor_spawn_runs_task() {
    let executor = TokioExecutor;
    let value = Arc::new(AtomicUsize::new(0));
    let value_for_task = Arc::clone(&value);

    executor.spawn(async move {
        value_for_task.fetch_add(1, Ordering::AcqRel);
    });

    tokio::task::yield_now().await;
    assert_eq!(value.load(Ordering::Acquire), 1);
}
