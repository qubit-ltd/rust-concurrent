/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`DirectExecutor`](qubit_concurrent::DirectExecutor).

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use qubit_concurrent::{DirectExecutor, Executor};

#[test]
fn test_direct_executor_execute_runs_inline() {
    let executor = DirectExecutor;
    let value = Arc::new(AtomicUsize::new(0));
    let value_for_task = Arc::clone(&value);

    executor.execute(Box::new(move || {
        value_for_task.fetch_add(1, Ordering::AcqRel);
    }));

    assert_eq!(value.load(Ordering::Acquire), 1);
}
