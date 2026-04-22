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
    use qubit_concurrent::{
        ArcMutex,
        DoubleCheckedLockExecutor,
        double_checked::ExecutionResult,
    };

    mod test_executor_builder {
        use super::*;

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
                .call_with(|value: &mut i32| Ok::<i32, std::io::Error>(*value))
                .get_result();

            assert!(matches!(result, ExecutionResult::Success(1)));
        }
    }
}
