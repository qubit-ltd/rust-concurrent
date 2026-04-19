/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`DirectExecutor`](qubit_concurrent::DirectExecutor).

use std::{
    io,
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
};

use qubit_concurrent::{
    BoxCallable,
    BoxRunnable,
    Callable,
    DirectExecutor,
    Executor,
    Runnable,
};

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

#[test]
fn test_executor_reexports_function_task_types() {
    let runnable: BoxRunnable<io::Error> = Runnable::into_box(|| Ok::<(), io::Error>(()));
    runnable.run().expect("re-exported runnable should run");

    let callable: BoxCallable<i32, io::Error> = Callable::into_box(|| Ok::<i32, io::Error>(42));
    assert_eq!(
        callable
            .call()
            .expect("re-exported callable should return a value"),
        42,
    );
}
