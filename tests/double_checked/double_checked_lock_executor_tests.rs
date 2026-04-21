/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{
            Arc,
            atomic::{
                AtomicBool,
                AtomicUsize,
                Ordering,
            },
        },
    };

    use qubit_concurrent::{
        ArcMutex,
        DoubleCheckedLockExecutor,
        double_checked::{
            ExecutionContext,
            ExecutionResult,
            ExecutorError,
        },
        lock::Lock,
        task::executor::Executor,
    };

    mod test_double_checked_lock_executor {
        use super::*;

        #[test]
        fn test_call_with_executes_reusable_task_with_mutable_data() {
            let data = ArcMutex::new(10);
            let active = Arc::new(AtomicBool::new(false));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when({
                    let active = active.clone();
                    move || !active.load(Ordering::Acquire)
                })
                .build();

            let first = executor
                .call_with(|value: &mut i32| {
                    *value += 5;
                    Ok::<i32, io::Error>(*value)
                })
                .get_result();
            let second = executor
                .call_with(|value: &mut i32| {
                    *value += 7;
                    Ok::<i32, io::Error>(*value)
                })
                .get_result();

            assert!(matches!(first, ExecutionResult::Success(15)));
            assert!(matches!(second, ExecutionResult::Success(22)));
            assert_eq!(data.read(|value| *value), 22);
        }

        #[test]
        fn test_execute_with_reports_condition_not_met_without_lock_mutation() {
            let data = ArcMutex::new(10);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when(|| false)
                .build();

            let result = executor
                .execute_with(|value: &mut i32| {
                    *value += 1;
                    Ok::<(), io::Error>(())
                })
                .get_result();

            assert!(matches!(result, ExecutionResult::ConditionNotMet));
            assert_eq!(data.read(|value| *value), 10);
        }

        #[test]
        fn test_call_and_execute_run_zero_argument_tasks_as_executor_api() {
            let data = ArcMutex::new(0);
            let counter = Arc::new(AtomicUsize::new(0));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when(|| true)
                .build();

            let value = executor
                .call(|| Ok::<i32, io::Error>(42))
                .get_result()
                .unwrap();
            let executed = executor
                .execute({
                    let counter = counter.clone();
                    move || {
                        counter.fetch_add(1, Ordering::AcqRel);
                        Ok::<(), io::Error>(())
                    }
                })
                .get_result();

            assert_eq!(value, 42);
            assert!(matches!(executed, ExecutionResult::Success(())));
            assert_eq!(counter.load(Ordering::Acquire), 1);
        }

        #[test]
        fn test_executor_trait_call_returns_execution_context() {
            let data = ArcMutex::new(0);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when(|| true)
                .build();

            let context: ExecutionContext<i32, io::Error> =
                Executor::call(&executor, || Ok::<i32, io::Error>(7));

            assert!(matches!(context.get_result(), ExecutionResult::Success(7)));
        }

        #[test]
        fn test_prepare_commit_runs_after_success() {
            let data = ArcMutex::new(10);
            let prepared = Arc::new(AtomicBool::new(false));
            let committed = Arc::new(AtomicBool::new(false));
            let rolled_back = Arc::new(AtomicBool::new(false));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when(|| true)
                .prepare({
                    let prepared = prepared.clone();
                    move || {
                        prepared.store(true, Ordering::Release);
                        Ok::<(), io::Error>(())
                    }
                })
                .rollback_prepare({
                    let rolled_back = rolled_back.clone();
                    move || {
                        rolled_back.store(true, Ordering::Release);
                        Ok::<(), io::Error>(())
                    }
                })
                .commit_prepare({
                    let committed = committed.clone();
                    move || {
                        committed.store(true, Ordering::Release);
                        Ok::<(), io::Error>(())
                    }
                })
                .build();

            let result = executor
                .call_with(|value: &mut i32| {
                    *value += 1;
                    Ok::<i32, io::Error>(*value)
                })
                .get_result();

            assert!(matches!(result, ExecutionResult::Success(11)));
            assert!(prepared.load(Ordering::Acquire));
            assert!(committed.load(Ordering::Acquire));
            assert!(!rolled_back.load(Ordering::Acquire));
        }

        #[test]
        fn test_prepare_rollback_runs_after_task_failure() {
            let data = ArcMutex::new(10);
            let rolled_back = Arc::new(AtomicBool::new(false));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when(|| true)
                .prepare(|| Ok::<(), io::Error>(()))
                .rollback_prepare({
                    let rolled_back = rolled_back.clone();
                    move || {
                        rolled_back.store(true, Ordering::Release);
                        Ok::<(), io::Error>(())
                    }
                })
                .build();

            let result = executor
                .call_with(|_value: &mut i32| Err::<i32, _>(io::Error::other("task failed")))
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::TaskFailed(_))
            ));
            assert!(rolled_back.load(Ordering::Acquire));
        }

        #[test]
        fn test_prepare_failure_skips_task() {
            let data = ArcMutex::new(10);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when(|| true)
                .prepare(|| Err::<(), _>(io::Error::other("prepare failed")))
                .build();

            let result = executor
                .call_with(|value: &mut i32| {
                    *value += 1;
                    Ok::<i32, io::Error>(*value)
                })
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::PrepareFailed(message))
                    if message == "prepare failed"
            ));
            assert_eq!(data.read(|value| *value), 10);
        }

        #[test]
        fn test_prepare_rollback_failure_replaces_result() {
            let data = ArcMutex::new(10);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when(|| true)
                .prepare(|| Ok::<(), io::Error>(()))
                .rollback_prepare(|| Err::<(), _>(io::Error::other("rollback failed")))
                .build();

            let result = executor
                .call_with(|_value: &mut i32| Err::<i32, _>(io::Error::other("task failed")))
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::PrepareRollbackFailed {
                    rollback,
                    ..
                }) if rollback == "rollback failed"
            ));
        }
    }
}
