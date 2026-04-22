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
            ExecutionResult,
            ExecutorError,
        },
        lock::Lock,
    };

    mod test_double_checked_lock_executor_builder {
        use super::*;

        fn increment_and_return_task(value: &mut i32) -> Result<i32, io::Error> {
            *value += 1;
            Ok(*value)
        }

        fn increment_unit_task(value: &mut i32) -> Result<(), io::Error> {
            *value += 1;
            Ok(())
        }

        #[test]
        fn test_logger_can_be_configured_in_each_builder_state() {
            let data = ArcMutex::new(1);
            let executor = DoubleCheckedLockExecutor::builder()
                .logger(log::Level::Info, "initial")
                .on(data)
                .logger(log::Level::Debug, "locked")
                .when(|| true)
                .logger(log::Level::Warn, "ready")
                .build();

            let result = executor
                .call_with(|value: &mut i32| Ok::<i32, io::Error>(*value))
                .get_result();

            assert!(matches!(result, ExecutionResult::Success(1)));
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
                .call_with(increment_and_return_task as fn(&mut i32) -> Result<i32, io::Error>)
                .get_result();

            assert!(matches!(result, ExecutionResult::Success(11)));
            assert!(prepared.load(Ordering::Acquire));
            assert!(committed.load(Ordering::Acquire));
            assert!(!rolled_back.load(Ordering::Acquire));
        }

        #[test]
        fn test_execute_with_prepare_commit_finalizes_unit_result() {
            let data = ArcMutex::new(10);
            let committed = Arc::new(AtomicBool::new(false));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when(|| true)
                .prepare(|| Ok::<(), io::Error>(()))
                .commit_prepare({
                    let committed = committed.clone();
                    move || {
                        committed.store(true, Ordering::Release);
                        Ok::<(), io::Error>(())
                    }
                })
                .build();

            let result = executor
                .execute_with(increment_unit_task as fn(&mut i32) -> Result<(), io::Error>)
                .get_result();

            assert!(matches!(result, ExecutionResult::Success(())));
            assert!(committed.load(Ordering::Acquire));
            assert_eq!(data.read(|value| *value), 11);
        }

        #[test]
        fn test_prepare_commit_failure_without_logger_replaces_success() {
            let data = ArcMutex::new(10);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when(|| true)
                .prepare(|| Ok::<(), io::Error>(()))
                .commit_prepare(|| Err::<(), _>(io::Error::other("commit failed")))
                .build();

            let result = executor
                .call_with(|value: &mut i32| Ok::<i32, io::Error>(*value))
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::PrepareCommitFailed(message))
                    if message == "commit failed"
            ));
        }

        #[test]
        fn test_prepare_commit_failure_with_logger_replaces_success() {
            let data = ArcMutex::new(10);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when(|| true)
                .logger(log::Level::Error, "condition not met")
                .prepare(|| Ok::<(), io::Error>(()))
                .commit_prepare(|| Err::<(), _>(io::Error::other("commit failed")))
                .build();

            let result = executor
                .call_with(|value: &mut i32| Ok::<i32, io::Error>(*value))
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::PrepareCommitFailed(message))
                    if message == "commit failed"
            ));
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
        fn test_second_condition_failure_after_prepare_rolls_back() {
            let data = ArcMutex::new(10);
            let checks = Arc::new(AtomicUsize::new(0));
            let rolled_back = Arc::new(AtomicBool::new(false));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when({
                    let checks = checks.clone();
                    move || checks.fetch_add(1, Ordering::AcqRel) == 0
                })
                .logger(log::Level::Info, "condition not met")
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
                .call_with(increment_and_return_task as fn(&mut i32) -> Result<i32, io::Error>)
                .get_result();

            assert!(matches!(result, ExecutionResult::ConditionNotMet));
            assert!(rolled_back.load(Ordering::Acquire));
            assert_eq!(data.read(|value| *value), 10);
            assert_eq!(checks.load(Ordering::Acquire), 2);
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
                .call_with(increment_and_return_task as fn(&mut i32) -> Result<i32, io::Error>)
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::PrepareFailed(message))
                    if message == "prepare failed"
            ));
            assert_eq!(data.read(|value| *value), 10);
        }

        #[test]
        fn test_prepare_failure_uses_configured_logger() {
            let data = ArcMutex::new(10);
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data.clone())
                .when(|| true)
                .logger(log::Level::Error, "condition not met")
                .prepare(|| Err::<(), _>(io::Error::other("prepare failed")))
                .build();

            let result = executor
                .call_with(increment_and_return_task as fn(&mut i32) -> Result<i32, io::Error>)
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

        #[test]
        fn test_prepare_rollback_failure_with_logger_replaces_condition_result() {
            let data = ArcMutex::new(10);
            let checks = Arc::new(AtomicUsize::new(0));
            let executor = DoubleCheckedLockExecutor::builder()
                .on(data)
                .when({
                    let checks = checks.clone();
                    move || checks.fetch_add(1, Ordering::AcqRel) == 0
                })
                .logger(log::Level::Error, "condition not met")
                .prepare(|| Ok::<(), io::Error>(()))
                .rollback_prepare(|| Err::<(), _>(io::Error::other("rollback failed")))
                .build();

            let result = executor
                .call_with(|value: &mut i32| Ok::<i32, io::Error>(*value))
                .get_result();

            assert!(matches!(
                result,
                ExecutionResult::Failed(ExecutorError::PrepareRollbackFailed {
                    original,
                    rollback,
                }) if original == "Condition not met" && rollback == "rollback failed"
            ));
        }
    }
}
