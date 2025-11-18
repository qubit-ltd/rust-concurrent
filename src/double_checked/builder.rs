/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Double-Checked Locking Execution Builder
//!
//! Provides a fluent API builder using the typestate pattern.
//!
//! # Author
//!
//! Haixing Hu

use std::{error::Error, marker::PhantomData};

use prism3_function::{
    BoxFunctionOnce, BoxMutatingFunctionOnce, BoxSupplierOnce, BoxTester, FunctionOnce,
    MutatingFunctionOnce, SupplierOnce, Tester,
};

use super::{
    states::{Conditioned, Configuring, Initial},
    ExecutionContext, ExecutionResult, LogConfig,
};
use crate::lock::Lock;

/// Execution builder (using typestate pattern)
///
/// This builder uses the type system to enforce the correct call sequence
/// at compile time.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the lock
/// * `L` - Lock type (implements the Lock<T> trait)
/// * `T` - Type of data protected by the lock
/// * `State` - Current state (Initial, Configuring, Conditioned)
///
/// # Author
///
/// Haixing Hu
pub struct ExecutionBuilder<'a, L, T, State = Initial>
where
    L: Lock<T>,
{
    lock: &'a L,
    logger: Option<LogConfig>,
    tester: Option<BoxTester>,
    prepare_action: Option<BoxSupplierOnce<Result<(), Box<dyn Error + Send + Sync>>>>,
    _phantom: PhantomData<(T, State)>,
}

// ============================================================================
// Initial state: Just created, can configure logger or set conditions
// ============================================================================

impl<'a, L, T> ExecutionBuilder<'a, L, T, Initial>
where
    L: Lock<T>,
{
    /// Creates a new execution builder
    ///
    /// # Arguments
    ///
    /// * `lock` - Reference to the lock object
    pub(super) fn new(lock: &'a L) -> Self {
        Self {
            lock,
            logger: None,
            tester: None,
            prepare_action: None,
            _phantom: PhantomData,
        }
    }

    /// Configures logging (optional)
    ///
    /// # State Transition
    ///
    /// Initial → Configuring
    ///
    /// # Arguments
    ///
    /// * `level` - Log level
    /// * `message` - Log message
    pub fn logger(mut self, level: log::Level, message: &str) -> ExecutionBuilder<'a, L, T, Configuring> {
        self.logger = Some(LogConfig {
            level,
            message: message.to_string(),
        });
        ExecutionBuilder {
            lock: self.lock,
            logger: self.logger,
            tester: self.tester,
            prepare_action: self.prepare_action,
            _phantom: PhantomData,
        }
    }

    /// Sets the test condition (required)
    ///
    /// # State Transition
    ///
    /// Initial → Conditioned
    ///
    /// # Arguments
    ///
    /// * `tester` - The test condition
    pub fn when<Tst>(mut self, tester: Tst) -> ExecutionBuilder<'a, L, T, Conditioned>
    where
        Tst: Tester + 'static,
    {
        self.tester = Some(tester.into_box());
        ExecutionBuilder {
            lock: self.lock,
            logger: self.logger,
            tester: self.tester,
            prepare_action: self.prepare_action,
            _phantom: PhantomData,
        }
    }
}

// ============================================================================
// Configuring state: Logger configured, can continue configuring or set
// conditions
// ============================================================================

impl<'a, L, T> ExecutionBuilder<'a, L, T, Configuring>
where
    L: Lock<T>,
{
    /// Continues configuring logging (can override previous configuration)
    ///
    /// # State Transition
    ///
    /// Configuring → Configuring
    ///
    /// # Arguments
    ///
    /// * `level` - Log level
    /// * `message` - Log message
    pub fn logger(mut self, level: log::Level, message: &str) -> Self {
        self.logger = Some(LogConfig {
            level,
            message: message.to_string(),
        });
        self
    }

    /// Sets the test condition (required)
    ///
    /// # State Transition
    ///
    /// Configuring → Conditioned
    ///
    /// # Arguments
    ///
    /// * `tester` - The test condition
    pub fn when<Tst>(mut self, tester: Tst) -> ExecutionBuilder<'a, L, T, Conditioned>
    where
        Tst: Tester + 'static,
    {
        self.tester = Some(tester.into_box());
        ExecutionBuilder {
            lock: self.lock,
            logger: self.logger,
            tester: self.tester,
            prepare_action: self.prepare_action,
            _phantom: PhantomData,
        }
    }
}

// ============================================================================
// Conditioned state: Condition set, can prepare and execute
// ============================================================================

impl<'a, L, T> ExecutionBuilder<'a, L, T, Conditioned>
where
    L: Lock<T>,
    T: 'static,
{
    /// Sets prepare action (optional, executed between first check and locking)
    ///
    /// # State Transition
    ///
    /// Conditioned → Conditioned
    ///
    /// # Arguments
    ///
    /// * `prepare_action` - Any type that implements
    ///   `SupplierOnce<Result<(), E>>`
    pub fn prepare<S, E>(mut self, prepare_action: S) -> Self
    where
        S: SupplierOnce<Result<(), E>> + 'static,
        E: Error + Send + Sync + 'static,
    {
        let boxed = prepare_action.into_box();
        self.prepare_action = Some(BoxSupplierOnce::new(move || {
            boxed.get().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
        }));
        self
    }

    /// Executes a read-only task (with return value)
    ///
    /// # Execution Flow
    ///
    /// 1. First condition check (outside lock)
    /// 2. Execute prepare action (if any)
    /// 3. Acquire lock
    /// 4. Second condition check (inside lock)
    /// 5. Execute task
    ///
    /// # State Transition
    ///
    /// Conditioned → ExecutionContext<R>
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that implements `FunctionOnce<T, Result<R, E>>`
    pub fn call<F, R, E>(self, task: F) -> ExecutionContext<R>
    where
        F: FunctionOnce<T, Result<R, E>> + 'static,
        E: Error + Send + Sync + 'static,
        R: 'static,
    {
        let task_boxed: BoxFunctionOnce<T, Result<R, E>> = task.into_box();
        let result = self.execute_with_read_lock(task_boxed);
        ExecutionContext::new(result)
    }

    /// Executes a read-write task (with return value)
    ///
    /// # State Transition
    ///
    /// Conditioned → ExecutionContext<R>
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that implements
    ///   `MutatingFunctionOnce<T, Result<R, E>>`
    pub fn call_mut<F, R, E>(self, task: F) -> ExecutionContext<R>
    where
        F: MutatingFunctionOnce<T, Result<R, E>> + 'static,
        E: Error + Send + Sync + 'static,
        R: 'static,
    {
        let task_boxed: BoxMutatingFunctionOnce<T, Result<R, E>> = task.into_box();
        let result = self.execute_with_write_lock(task_boxed);
        ExecutionContext::new(result)
    }

    /// Executes a read-only task (without return value)
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that implements `FunctionOnce<T, Result<(), E>>`
    pub fn execute<F, E>(self, task: F) -> ExecutionContext<()>
    where
        F: FunctionOnce<T, Result<(), E>> + 'static,
        E: Error + Send + Sync + 'static,
    {
        self.call(task)
    }

    /// Executes a read-write task (without return value)
    ///
    /// # Arguments
    ///
    /// * `task` - Any type that implements
    ///   `MutatingFunctionOnce<T, Result<(), E>>`
    pub fn execute_mut<F, E>(self, task: F) -> ExecutionContext<()>
    where
        F: MutatingFunctionOnce<T, Result<(), E>> + 'static,
        E: Error + Send + Sync + 'static,
    {
        self.call_mut(task)
    }

    // ========================================================================
    // Internal helper methods
    // ========================================================================

    fn execute_with_read_lock<R, E>(mut self, task: BoxFunctionOnce<T, Result<R, E>>) -> ExecutionResult<R>
    where
        E: Error + Send + Sync + 'static,
    {
        // 第一次检查（锁外）
        let tester = self.tester.take().expect("Tester must be set in Conditioned state");
        if !tester.test() {
            if let Some(ref log_config) = self.logger {
                log::log!(log_config.level, "{}", log_config.message);
            }
            return ExecutionResult::unmet();
        }

        // 执行准备动作
        if let Some(prepare_action) = self.prepare_action.take() {
            if let Err(e) = prepare_action.get() {
                log::error!("Prepare action failed: {}", e);
                return ExecutionResult::fail_with_box(e);
            }
        }

        // 获取锁并执行
        let logger = self.logger;
        let handle_condition_not_met = move || {
            if let Some(ref log_config) = logger {
                log::log!(log_config.level, "{}", log_config.message);
            }
        };

        self.lock.read(|data| {
            // 第二次检查（锁内）
            if !tester.test() {
                handle_condition_not_met();
                return ExecutionResult::unmet();
            }
            // 执行任务
            task.apply(data)
                .map_or_else(ExecutionResult::fail, ExecutionResult::succeed)
        })
    }

    fn execute_with_write_lock<R, E>(mut self, task: BoxMutatingFunctionOnce<T, Result<R, E>>) -> ExecutionResult<R>
    where
        E: Error + Send + Sync + 'static,
    {
        // 第一次检查（锁外）
        let tester = self.tester.take().expect("Tester must be set in Conditioned state");
        if !tester.test() {
            if let Some(ref log_config) = self.logger {
                log::log!(log_config.level, "{}", log_config.message);
            }
            return ExecutionResult::unmet();
        }

        // 执行准备动作
        if let Some(prepare_action) = self.prepare_action.take() {
            if let Err(e) = prepare_action.get() {
                log::error!("Prepare action failed: {}", e);
                return ExecutionResult::fail_with_box(e);
            }
        }

        // 获取锁并执行
        let logger = self.logger;
        let handle_condition_not_met = move || {
            if let Some(ref log_config) = logger {
                log::log!(log_config.level, "{}", log_config.message);
            }
        };

        self.lock.write(|data| {
            // 第二次检查（锁内）
            if !tester.test() {
                handle_condition_not_met();
                return ExecutionResult::unmet();
            }
            // 执行任务
            task.apply(data)
                .map_or_else(ExecutionResult::fail, ExecutionResult::succeed)
        })
    }
}
