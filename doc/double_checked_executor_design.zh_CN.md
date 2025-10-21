# 双重检查锁执行器（Double-Checked Lock Executor）设计文档

## 1. 概述

本文档描述了如何将 Java 版本的 `DoubleCheckedLockExecutor` 移植到 Rust，充分利用 Rust 的类型系统、所有权模型和并发安全特性。

### 1.1 目标

- 提供与 Java 版本功能等价的 Rust 实现
- 利用 Rust 的编译期保证实现更强的线程安全
- 集成现有的 `prism3-rust-function` 和 `prism3-rust-clock` 组件
- 提供符合 Rust 习惯的 API 设计

### 1.2 适用场景

双重检查锁执行器适用于以下场景：

- 共享状态会在运行时发生变化（例如：服务的启动/停止状态）
- 大部分情况下条件满足，只有少数情况下条件不满足
- 需要在条件满足时执行需要同步保护的操作
- 需要最小化锁竞争，提高并发性能

## 2. Java 版本功能分析

### 2.1 核心功能

1. **双重检查锁模式**
   - 第一次检查：在锁外快速失败
   - 获取锁
   - 第二次检查：在锁内确认条件
   - 执行任务

2. **条件测试器**
   - `BooleanSupplier tester`：用于检查执行条件（Java）
   - Rust 移植：使用 `ArcTester`（来自 `prism3-rust-function`）
   - 依赖的共享状态必须通过线程安全类型（如 `Arc<AtomicBool>`、`Arc<Mutex<T>>`）保证可见性

3. **灵活的错误处理**
   - 可选的日志记录（logger、level、message）
   - 可选的异常抛出（errorSupplier）
   - 支持无栈异常以优化性能

4. **回滚机制**
   - `outsideAction`：锁外准备操作
   - `rollbackAction`：失败时的回滚操作
   - 自动处理回滚异常

5. **多种任务类型**
   - `execute()`：无返回值的任务（Runnable）
   - `call()`：有返回值的任务（Callable）
   - `executeIo()`：可能抛出 IOException 的任务
   - `callIo()`：可能抛出 IOException 且有返回值的任务

### 2.2 关键设计特性

- **Builder 模式**：流式 API 构建执行器
- **异常工厂**：避免并发场景下复用异常导致的栈覆盖问题
- **锁抽象**：支持普通 Lock、ReadLock、WriteLock
- **Result 包装**：封装成功/失败状态和返回值

## 3. Rust 移植方案

### 3.1 总体架构

```
DoubleCheckedLockExecutor
├── 核心配置
│   ├── tester: 条件测试函数
│   ├── logger: 日志配置（可选）
│   └── error_supplier: 错误工厂（可选）
├── Builder 模式
│   └── 流式 API 构建
└── 执行方法
    ├── execute/call: 基础版本
    └── execute_with_rollback/call_with_rollback: 带回滚版本
```

### 3.2 模块结构

```
prism3-rust-concurrent/
├── src/
│   ├── lib.rs
│   ├── executor.rs           # 执行器接口抽象 trait（Runnable、Callable、Executor 等）
│   ├── lock.rs              # 锁包装器（ArcMutex、ArcRwLock、ArcAsyncMutex 等）
│   ├── double_checked_lock/   # 新增：双重检查锁执行器模块
│   │   ├── mod.rs           # 模块导出
│   │   ├── executor.rs      # 双重检查锁执行器实现
│   │   ├── builder.rs       # Builder 实现
│   │   ├── config.rs        # 配置结构体
│   │   ├── error.rs         # 错误类型定义
│   │   └── result.rs        # ExecutionResult 类型
│   └── traits/               # 可选：锁的抽象 trait
│       └── lock_trait.rs    # 锁的抽象 trait（可选）
├── tests/
│   └── double_checked_lock_tests.rs
├── examples/
│   └── double_checked_lock_demo.rs
└── doc/
    └── double_checked_executor_design.zh_CN.md  # 本文档
```

**现有代码说明：**
- `executor.rs` - 定义了执行器相关的抽象 trait，类似 JDK 的 Executor 接口
- `lock.rs` - 提供了同步和异步锁的包装器，简化锁的使用
- 新增的 `double_checked_lock/` 模块将实现双重检查锁执行器功能

## 4. 类型系统设计

### 4.1 核心结构体

```rust
use prism3_rust_function::ArcTester;

/// 双重检查锁执行器
///
/// 泛型参数：
/// - `E`: 条件不满足时返回的错误类型
pub struct DoubleCheckedLockExecutor<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// 条件测试器 - 测试执行条件是否满足
    /// 注意：此测试器依赖的共享状态必须通过 Arc<AtomicXxx> 或 Arc<Mutex<T>> 等
    /// 线程安全类型来保证可见性
    tester: ArcTester,

    /// 日志配置（可选）
    logger: Option<LogConfig>,

    /// 错误工厂 - 用于创建错误实例（可选）
    error_supplier: Option<Arc<dyn Fn() -> E + Send + Sync>>,

    /// 执行器配置
    config: ExecutorConfig,
}

/// 日志配置
pub struct LogConfig {
    /// 日志级别
    level: log::Level,

    /// 日志消息
    message: String,
}

/// 执行器配置
pub struct ExecutorConfig {
    /// 是否启用性能度量
    enable_metrics: bool,

    /// 是否禁用错误回溯（用于高性能场景）
    disable_backtrace: bool,
}
```

#### 为何使用 `ArcTester` 而不是 `BoxTester`？

`prism3-rust-function` 提供了三种 Tester 实现：`BoxTester`、`RcTester` 和 `ArcTester`。在 `DoubleCheckedLockExecutor` 中必须使用 `ArcTester`，原因如下：

**1. 线程安全保证（关键因素）**

```rust
// BoxTester - 不保证线程安全
pub struct BoxTester {
    func: Box<dyn Fn() -> bool>,  // ❌ 没有 Send + Sync 约束
}

// ArcTester - 编译期保证线程安全
pub struct ArcTester {
    func: Arc<dyn Fn() -> bool + Send + Sync>,  // ✅ 强制 Send + Sync
}
```

`ArcTester` 的 `Send + Sync` 约束确保在编译期捕获线程安全问题，防止意外捕获非线程安全的数据（如 `Rc`、`RefCell`）。

**2. 跨线程使用需求**

```rust
// 典型使用场景
pub struct Service {
    executor: DoubleCheckedLockExecutor<ServiceError>,  // 需要 Send
}

// 在多线程中使用
let service = Service::new();
std::thread::spawn(move || {
    service.set_pool_size(10);  // executor 必须实现 Send
});
```

**3. Arc 共享场景**

```rust
// 多线程共享同一个 Service
let service = Arc::new(Service::new());  // Service 必须实现 Sync

let s1 = service.clone();
let t1 = std::thread::spawn(move || s1.set_pool_size(10));

let s2 = service.clone();
let t2 = std::thread::spawn(move || s2.set_cache_size(20));
```

**对比总结**

| 特性 | BoxTester | ArcTester | 需求 |
|------|-----------|-----------|------|
| **Send** | ❌ 不保证 | ✅ 保证 | ✅ 必需 |
| **Sync** | ❌ 不保证 | ✅ 保证 | ✅ 必需 |
| **编译期检查** | ❌ 无 | ✅ 有 | ✅ 关键 |
| **Clone** | ❌ 不支持 | ✅ 支持 | ⚖️ 附带好处 |

**注意**：选择 `ArcTester` 的核心原因是**编译期线程安全保证**，而非克隆能力。实际使用中，`DoubleCheckedLockExecutor` 作为结构体字段只创建一次，不需要克隆。`Arc` 的引用计数开销在这个场景中完全可以接受。

### 4.2 结果类型

```rust
/// 任务执行结果
///
/// 类似 Java 版本的 `Result<T>` 类，但为了避免与 Rust 标准库的 `Result` 混淆，
/// 命名为 `ExecutionResult`
pub struct ExecutionResult<T> {
    /// 执行是否成功
    pub success: bool,

    /// 成功时的返回值（仅当 success = true 时有值）
    pub value: Option<T>,

    /// 失败时的错误信息（仅当 success = false 时有值）
    pub error: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl<T> ExecutionResult<T> {
    /// 创建成功结果
    pub fn succeed(value: T) -> Self {
        Self {
            success: true,
            value: Some(value),
            error: None,
        }
    }

    /// 创建失败结果
    pub fn fail() -> Self {
        Self {
            success: false,
            value: None,
            error: None,
        }
    }

    /// 创建带错误信息的失败结果
    pub fn fail_with_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            success: false,
            value: None,
            error: Some(Box::new(error)),
        }
    }
}
```

### 4.3 错误类型

```rust
/// 执行器错误类型
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    /// 条件不满足
    #[error("Double-checked lock condition not met")]
    ConditionNotMet,

    /// 条件不满足，带自定义消息
    #[error("Double-checked lock condition not met: {0}")]
    ConditionNotMetWithMessage(String),

    /// 任务执行失败
    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    /// 回滚操作失败
    #[error("Rollback failed: original error = {original}, rollback error = {rollback}")]
    RollbackFailed {
        original: String,
        rollback: String,
    },

    /// 锁中毒（Mutex/RwLock poison）
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}

/// Builder 错误类型
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    /// 缺少必需的 tester 参数
    #[error("Tester function is required")]
    MissingTester,
}
```

## 5. 锁抽象设计

### 5.1 方案选择

**推荐方案**：不引入额外的 trait，直接为标准库的锁类型提供实现

原因：
- Rust 标准库的锁类型（`Mutex`、`RwLock`）已经足够好用
- 避免过度抽象增加复杂度
- 可以直接利用锁卫士（Guard）的 RAII 特性

### 5.2 支持的锁类型

```rust
// 支持以下锁类型：
use std::sync::{Mutex, RwLock};
use parking_lot::{Mutex as ParkingLotMutex, RwLock as ParkingLotRwLock};

// 执行器方法接受 MutexGuard 或 RwLockReadGuard/RwLockWriteGuard
// 调用方负责获取锁卫士
```

### 5.3 使用模式

```rust
// 方式1: 传入锁卫士（推荐）
let guard = mutex.lock().unwrap();
executor.execute_with_guard(guard, || { /* task */ });

// 方式2: 传入锁引用，内部获取（便捷方法）
executor.execute_mutex(&mutex, || { /* task */ });
executor.execute_rwlock_write(&rwlock, || { /* task */ });
executor.execute_rwlock_read(&rwlock, || { /* task */ });
```

## 6. API 设计

### 6.1 Builder API

```rust
use prism3_rust_function::{ArcTester, Tester};

impl<E> DoubleCheckedLockExecutor<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// 创建 Builder
    pub fn builder() -> Builder<E> {
        Builder::default()
    }
}

/// Builder 构建器
pub struct Builder<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    tester: Option<ArcTester>,
    logger: Option<LogConfig>,
    error_supplier: Option<Arc<dyn Fn() -> E + Send + Sync>>,
    config: ExecutorConfig,
}

impl<E> Builder<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// 设置条件测试器（必需）
    ///
    /// 接受一个 `ArcTester` 实例，用于测试执行条件是否满足。
    ///
    /// **重要**：测试器依赖的共享状态必须通过 `Arc<AtomicBool>`、`Arc<Mutex<T>>`
    /// 或 `Arc<RwLock<T>>` 等线程安全类型来保证跨线程可见性
    ///
    /// # 参数
    ///
    /// * `tester` - 条件测试器
    ///
    /// # 示例
    ///
    /// ```rust
    /// use prism3_rust_function::ArcTester;
    /// use std::sync::{Arc, RwLock};
    ///
    /// let state = Arc::new(RwLock::new(State::Running));
    /// let state_clone = state.clone();
    ///
    /// let executor = DoubleCheckedLockExecutor::builder()
    ///     .tester(ArcTester::new(move || {
    ///         matches!(*state_clone.read().unwrap(), State::Running)
    ///     }))
    ///     .build()?;
    /// ```
    pub fn tester(mut self, tester: ArcTester) -> Self {
        self.tester = Some(tester);
        self
    }

    /// 设置条件测试闭包（便捷方法）
    ///
    /// 接受一个闭包，内部自动创建 `ArcTester`。
    ///
    /// # 参数
    ///
    /// * `f` - 测试条件的闭包
    ///
    /// # 示例
    ///
    /// ```rust
    /// use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    ///
    /// let running = Arc::new(AtomicBool::new(true));
    /// let running_clone = running.clone();
    ///
    /// let executor = DoubleCheckedLockExecutor::builder()
    ///     .tester_fn(move || running_clone.load(Ordering::Acquire))
    ///     .build()?;
    /// ```
    pub fn tester_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.tester = Some(ArcTester::new(f));
        self
    }

    /// 设置日志记录器（可选）
    pub fn logger(mut self, level: log::Level, message: impl Into<String>) -> Self {
        self.logger = Some(LogConfig {
            level,
            message: message.into(),
        });
        self
    }

    /// 设置错误工厂（可选）
    pub fn error_supplier<F>(mut self, f: F) -> Self
    where
        F: Fn() -> E + Send + Sync + 'static,
    {
        self.error_supplier = Some(Arc::new(f));
        self
    }

    /// 设置错误消息（便捷方法，用于简单场景）
    pub fn error_message(mut self, message: impl Into<String>) -> Self
    where
        E: From<ExecutorError>,
    {
        let msg = message.into();
        self.error_supplier = Some(Arc::new(move || {
            E::from(ExecutorError::ConditionNotMetWithMessage(msg.clone()))
        }));
        self
    }

    /// 启用性能度量（可选）
    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    /// 禁用错误回溯以提升性能（可选）
    pub fn disable_backtrace(mut self, disable: bool) -> Self {
        self.config.disable_backtrace = disable;
        self
    }

    /// 构建执行器
    pub fn build(self) -> Result<DoubleCheckedLockExecutor<E>, BuilderError> {
        let tester = self.tester.ok_or(BuilderError::MissingTester)?;

        Ok(DoubleCheckedLockExecutor {
            tester,
            logger: self.logger,
            error_supplier: self.error_supplier,
            config: self.config,
        })
    }
}
```

### 6.2 执行方法

```rust
impl<E> DoubleCheckedLockExecutor<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    // ==================== 基础版本 ====================

    /// 执行无返回值的任务
    ///
    /// # 参数
    /// - `mutex`: 互斥锁引用
    /// - `task`: 要执行的任务，返回 `Result<(), Box<dyn Error>>`
    ///
    /// # 返回值
    /// 返回 `ExecutionResult<()>`，其中 `success` 字段指示是否成功执行
    pub fn execute_mutex<T, F>(
        &self,
        mutex: &Mutex<T>,
        task: F,
    ) -> ExecutionResult<()>
    where
        F: FnOnce(&mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        self.call_mutex(mutex, |t| {
            task(t)?;
            Ok(())
        })
    }

    /// 执行有返回值的任务
    ///
    /// # 参数
    /// - `mutex`: 互斥锁引用
    /// - `task`: 要执行的任务，返回 `Result<R, Box<dyn Error>>`
    ///
    /// # 返回值
    /// 返回 `ExecutionResult<R>`，包含任务的返回值（如果成功）
    pub fn call_mutex<T, F, R>(
        &self,
        mutex: &Mutex<T>,
        task: F,
    ) -> ExecutionResult<R>
    where
        F: FnOnce(&mut T) -> Result<R, Box<dyn std::error::Error + Send + Sync>>,
    {
        // 第一次检查：锁外快速失败
        if !(self.tester)() {
            self.handle_condition_not_met();
            return ExecutionResult::fail();
        }

        // 获取锁
        let mut guard = match mutex.lock() {
            Ok(g) => g,
            Err(e) => {
                log::error!("Failed to acquire lock: {}", e);
                return ExecutionResult::fail_with_error(
                    ExecutorError::LockPoisoned(e.to_string())
                );
            }
        };

        // 第二次检查：锁内再次确认
        if !(self.tester)() {
            drop(guard); // 显式释放锁
            self.handle_condition_not_met();
            return ExecutionResult::fail();
        }

        // 执行任务
        match task(&mut guard) {
            Ok(value) => ExecutionResult::succeed(value),
            Err(e) => ExecutionResult::fail_with_error(
                ExecutorError::TaskFailed(e.to_string())
            ),
        }
    }

    /// 执行无返回值的任务（RwLock 写锁）
    pub fn execute_rwlock_write<T, F>(
        &self,
        rwlock: &RwLock<T>,
        task: F,
    ) -> ExecutionResult<()>
    where
        F: FnOnce(&mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        // 实现类似 execute_mutex
        todo!()
    }

    /// 执行有返回值的任务（RwLock 读锁）
    pub fn call_rwlock_read<T, F, R>(
        &self,
        rwlock: &RwLock<T>,
        task: F,
    ) -> ExecutionResult<R>
    where
        F: FnOnce(&T) -> Result<R, Box<dyn std::error::Error + Send + Sync>>,
    {
        // 实现类似 call_mutex，但使用 read() 而非 lock()
        todo!()
    }

    // ==================== 带回滚版本 ====================

    /// 执行无返回值的任务，并提供回滚机制
    ///
    /// # 执行流程
    /// 1. 检查条件是否满足，若不满足则失败
    /// 2. 若条件满足，先执行 `outside_action`
    /// 3. 然后获取锁，再次检查条件：
    ///    - 若不满足，则释放锁后执行 `rollback_action` 并失败
    ///    - 若满足，则执行 `task`
    /// 4. 若 `task` 执行时抛出异常，则释放锁后执行 `rollback_action`
    ///
    /// # 参数
    /// - `mutex`: 互斥锁引用
    /// - `task`: 要执行的核心任务
    /// - `outside_action`: 锁外准备操作
    /// - `rollback_action`: 失败时的回滚操作
    ///
    /// # 死锁警告
    /// `outside_action` 在获取锁**之前**执行，因此**禁止**在此操作中尝试获取
    /// 相同的锁或任何可能形成锁环的其他锁，否则会导致死锁！
    pub fn execute_with_rollback_mutex<T, F, O, R>(
        &self,
        mutex: &Mutex<T>,
        task: F,
        outside_action: O,
        rollback_action: R,
    ) -> ExecutionResult<()>
    where
        F: FnOnce(&mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
        O: FnOnce() -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
        R: FnOnce() -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        self.call_with_rollback_mutex(
            mutex,
            |t| {
                task(t)?;
                Ok(())
            },
            outside_action,
            rollback_action,
        )
    }

    /// 执行有返回值的任务，并提供回滚机制
    pub fn call_with_rollback_mutex<T, F, O, R, V>(
        &self,
        mutex: &Mutex<T>,
        task: F,
        outside_action: O,
        rollback_action: R,
    ) -> ExecutionResult<V>
    where
        F: FnOnce(&mut T) -> Result<V, Box<dyn std::error::Error + Send + Sync>>,
        O: FnOnce() -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
        R: FnOnce() -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        // 第一次检查
        if !(self.tester)() {
            self.handle_condition_not_met();
            return ExecutionResult::fail();
        }

        // 执行锁外操作
        if let Err(e) = outside_action() {
            log::error!("Outside action failed: {}", e);
            return ExecutionResult::fail_with_error(
                ExecutorError::TaskFailed(e.to_string())
            );
        }

        // 获取锁
        let mut guard = match mutex.lock() {
            Ok(g) => g,
            Err(e) => {
                self.run_rollback(&rollback_action, None);
                return ExecutionResult::fail_with_error(
                    ExecutorError::LockPoisoned(e.to_string())
                );
            }
        };

        // 第二次检查
        if !(self.tester)() {
            drop(guard);
            self.handle_condition_not_met();
            self.run_rollback(&rollback_action, None);
            return ExecutionResult::fail();
        }

        // 执行任务
        match task(&mut guard) {
            Ok(value) => ExecutionResult::succeed(value),
            Err(e) => {
                drop(guard);
                let error_msg = e.to_string();
                self.run_rollback(&rollback_action, Some(&error_msg));
                ExecutionResult::fail_with_error(
                    ExecutorError::TaskFailed(error_msg)
                )
            }
        }
    }

    // ==================== 内部辅助方法 ====================

    /// 处理条件不满足的情况
    fn handle_condition_not_met(&self) {
        // 记录日志
        if let Some(ref log_config) = self.logger {
            log::log!(log_config.level, "{}", log_config.message);
        }

        // 抛出错误（如果配置了 error_supplier）
        // 注意：Rust 中不能直接抛出异常，这里通过返回值传递错误
        // 实际实现中，可以将 error_supplier 的结果存储到 ExecutionResult 中
    }

    /// 执行回滚操作
    fn run_rollback<R>(
        &self,
        rollback_action: R,
        original_error: Option<&str>,
    ) where
        R: FnOnce() -> Result<(), Box<dyn std::error::Error + Send + Sync>>,
    {
        if let Err(e) = rollback_action() {
            if let Some(original) = original_error {
                log::warn!(
                    "Rollback failed during error recovery: {}. Original error: {}",
                    e,
                    original
                );
            } else {
                log::error!("Rollback failed: {}", e);
            }
        }
    }
}
```

### 6.3 集成 prism3-rust-clock

```rust
use prism3_rust_clock::meter::TimeMeter;

impl<E> DoubleCheckedLockExecutor<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// 执行任务并度量时间
    pub fn call_with_metrics_mutex<T, F, R>(
        &self,
        mutex: &Mutex<T>,
        task: F,
        meter: &mut TimeMeter,
    ) -> ExecutionResult<R>
    where
        F: FnOnce(&mut T) -> Result<R, Box<dyn std::error::Error + Send + Sync>>,
    {
        meter.start();
        let result = self.call_mutex(mutex, task);
        meter.stop();
        result
    }
}
```

### 6.4 集成 prism3-rust-function

```rust
use prism3_rust_function::{Predicate, Supplier, Consumer};

impl<E> Builder<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// 使用 Predicate trait 设置测试函数
    pub fn tester_predicate<P>(mut self, predicate: P) -> Self
    where
        P: Predicate<Input = ()> + Send + Sync + 'static,
    {
        self.tester = Some(Arc::new(move || predicate.test(&())));
        self
    }
}
```

## 7. 使用示例

### 7.1 基础用法

```rust
use std::sync::{Arc, RwLock};
use prism3_rust_concurrent::DoubleCheckedLockExecutor;

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Stopped,
    Running,
    Stopping,
}

struct Service {
    state: Arc<RwLock<State>>,
    pool_size: i32,
    cache_size: i32,
    executor: DoubleCheckedLockExecutor<ServiceError>,
}

#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error("Service is not running")]
    NotRunning,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl Service {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let state = Arc::new(RwLock::new(State::Stopped));
        let state_clone = state.clone();

        // 构建执行器
        let executor = DoubleCheckedLockExecutor::builder()
            .tester(move || {
                if let Ok(guard) = state_clone.read() {
                    *guard == State::Running
                } else {
                    false
                }
            })
            .logger(
                log::Level::Error,
                "Cannot change states while the service is not running",
            )
            .error_supplier(|| ServiceError::NotRunning)
            .build()?;

        Ok(Self {
            state,
            pool_size: 0,
            cache_size: 0,
            executor,
        })
    }

    /// 设置线程池大小
    pub fn set_pool_size(&mut self, size: i32) -> Result<(), Box<dyn std::error::Error>> {
        // 创建一个包含 self 字段的 Mutex 来保护修改操作
        // 实际使用中，可能需要更细粒度的锁设计
        let data = Arc::new(Mutex::new((self.pool_size, self.cache_size)));

        let result = self.executor.execute_mutex(&data, move |fields| {
            if size <= 0 {
                return Err(Box::new(ServiceError::InvalidParameter(
                    "pool size must be positive".to_string()
                )));
            }
            fields.0 = size;
            Ok(())
        });

        if result.success {
            if let Ok(guard) = data.lock() {
                self.pool_size = guard.0;
            }
            Ok(())
        } else {
            Err("Failed to set pool size".into())
        }
    }

    /// 获取线程池大小
    pub fn get_pool_size(&self) -> Option<i32> {
        let data = Arc::new(RwLock::new(self.pool_size));

        let result = self.executor.call_rwlock_read(&data, |&pool_size| {
            Ok(pool_size)
        });

        if result.success {
            result.value
        } else {
            None
        }
    }
}
```

### 7.2 带回滚机制的用法

```rust
use std::sync::{Arc, Mutex};

struct ResourceManager {
    state: Arc<AtomicBool>,
    resources: Arc<Mutex<Vec<Resource>>>,
    executor: DoubleCheckedLockExecutor<ManagerError>,
}

impl ResourceManager {
    pub fn allocate_resource(&self) -> Result<Resource, Box<dyn std::error::Error>> {
        let mut temp_resource = None;

        let result = self.executor.call_with_rollback_mutex(
            &self.resources,
            |resources| {
                // 锁内操作：添加资源
                let resource = temp_resource.take().unwrap();
                resources.push(resource.clone());
                Ok(resource)
            },
            || {
                // 锁外操作：预分配资源
                temp_resource = Some(Resource::allocate()?);
                Ok(())
            },
            || {
                // 回滚操作：释放预分配的资源
                if let Some(resource) = temp_resource.take() {
                    resource.deallocate();
                }
                Ok(())
            },
        );

        result.value.ok_or_else(|| "Failed to allocate resource".into())
    }
}
```

### 7.3 集成时间度量

```rust
use prism3_rust_clock::meter::TimeMeter;

let mut meter = TimeMeter::new();

let result = executor.call_with_metrics_mutex(
    &mutex,
    |data| {
        // 执行耗时操作
        expensive_operation(data)?;
        Ok(42)
    },
    &mut meter,
);

println!("Execution time: {:?}", meter.elapsed());
```

## 8. Java vs Rust 关键差异对照

| 特性 | Java 版本 | Rust 版本 | 说明 |
|-----|---------|---------|------|
| **异常处理** | 异常（Exception） | `Result<T, E>` | Rust 使用显式错误处理，更安全 |
| **锁类型** | `Lock` 接口 | `Mutex<T>` / `RwLock<T>` | Rust 的锁直接保护数据 |
| **锁卫士** | 手动 lock/unlock | RAII Guard | Rust 自动释放锁，避免忘记 unlock |
| **闭包** | Lambda 表达式 | `Fn` / `FnOnce` / `FnMut` | Rust 需要明确所有权和生命周期 |
| **Volatile** | `volatile` 字段 | `Arc<AtomicXxx>` / `Arc<Mutex<T>>` | Rust 通过类型系统保证线程安全 |
| **线程安全** | `synchronized` / `volatile` | `Send` + `Sync` trait | 编译期检查，零运行时开销 |
| **日志** | SLF4J | `log` / `tracing` crate | Rust 生态标准 |
| **Builder** | 可选参数 | `Option<T>` | 必须显式处理 None 情况 |
| **错误栈** | `fillInStackTrace()` | `std::backtrace` | Rust 1.65+ 稳定 |
| **泛型** | 类型擦除 | 单态化 | Rust 保留类型信息，性能更好 |

## 9. 重要注意事项

### 9.1 Volatile 替代方案

Java 的 `volatile` 在 Rust 中没有直接对应物，需要使用以下方案：

```rust
// ✅ 推荐方案1：AtomicBool（适用于简单布尔状态）
use std::sync::atomic::{AtomicBool, Ordering};
let state = Arc::new(AtomicBool::new(false));
builder.tester(move || state.load(Ordering::Acquire))

// ✅ 推荐方案2：Arc<Mutex<T>>（适用于复杂状态）
let state = Arc::new(Mutex::new(State::Running));
builder.tester(move || {
    matches!(*state.lock().unwrap(), State::Running)
})

// ✅ 推荐方案3：Arc<RwLock<T>>（读多写少场景）
let state = Arc::new(RwLock::new(State::Running));
builder.tester(move || {
    matches!(*state.read().unwrap(), State::Running)
})

// ❌ 错误方案：普通 Rc<RefCell<T>>（不是线程安全的）
let state = Rc::new(RefCell::new(State::Running)); // 编译错误！
```

### 9.2 死锁预防

在使用 `execute_with_rollback` 或 `call_with_rollback` 时：

- ✅ `outside_action` 中可以进行文件 I/O、网络请求等无锁操作
- ✅ `outside_action` 中可以获取**不同**的锁（注意锁顺序）
- ❌ `outside_action` 中**禁止**获取 `mutex/rwlock` 参数指定的锁
- ❌ `outside_action` 中**禁止**获取可能形成锁环的其他锁

### 9.3 性能考虑

1. **高并发失败场景**：如果预期条件检查会频繁失败，考虑：
   - 使用 `disable_backtrace(true)` 禁用回溯
   - 避免使用 `error_supplier`，仅通过返回值判断

2. **锁竞争**：任务应尽量简短，避免在锁内执行耗时操作

3. **日志开销**：在性能敏感场景，考虑使用条件编译或运行时开关控制日志

### 9.4 错误处理最佳实践

```rust
// ✅ 推荐：使用 ExecutionResult 检查成功状态
let result = executor.call_mutex(&mutex, task);
if result.success {
    let value = result.value.unwrap();
    // 使用 value
} else {
    if let Some(error) = result.error {
        log::error!("Execution failed: {}", error);
    }
}

// ✅ 推荐：转换为标准 Result
impl<T> ExecutionResult<T> {
    pub fn into_result(self) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        if self.success {
            Ok(self.value.unwrap())
        } else {
            Err(self.error.unwrap_or_else(|| "Unknown error".into()))
        }
    }
}

// 使用
executor.call_mutex(&mutex, task).into_result()?;
```

## 10. 实施计划

### 10.1 第一阶段：基础实现

- [ ] 创建模块结构
- [ ] 实现 `ExecutionResult` 类型
- [ ] 实现 `ExecutorError` 和 `BuilderError`
- [ ] 实现 `LogConfig` 和 `ExecutorConfig`
- [ ] 实现 `DoubleCheckedLockExecutor` 结构体
- [ ] 实现 `Builder` 及其方法

### 10.2 第二阶段：核心功能

- [ ] 实现 `execute_mutex()` 方法
- [ ] 实现 `call_mutex()` 方法
- [ ] 实现 `execute_rwlock_write()` 方法
- [ ] 实现 `call_rwlock_read()` 方法
- [ ] 实现内部辅助方法

### 10.3 第三阶段：高级功能

- [ ] 实现带回滚机制的方法
- [ ] 集成 `prism3-rust-clock` 的 `TimeMeter`
- [ ] 集成 `prism3-rust-function` 的 trait
- [ ] 实现 `parking_lot` 锁的支持（可选）

### 10.4 第四阶段：测试和文档

- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 编写并发测试
- [ ] 编写使用示例
- [ ] 编写 API 文档注释
- [ ] 编写 README

### 10.5 第五阶段：优化和完善

- [ ] 性能基准测试
- [ ] 代码覆盖率测试
- [ ] 错误处理完善
- [ ] 日志输出优化
- [ ] 用户反馈收集

## 11. 开放问题

### 11.1 待讨论的设计选择

1. **锁抽象层级**
   - 是否需要引入 `LockTrait` 来统一不同锁类型？
   - 还是直接为每种锁类型提供专门的方法？

2. **错误类型设计**
   - 是否使用 `anyhow` 或 `eyre` 简化错误处理？
   - 还是坚持使用 `thiserror` 提供精确的错误类型？

3. **异步支持**
   - 是否需要提供 async 版本（`Tokio::Mutex`、`Tokio::RwLock`）？
   - 如果需要，是否作为单独的类型还是通过特性门控？

4. **日志框架**
   - 使用 `log` facade 还是直接依赖 `tracing`？
   - 是否支持自定义日志后端？

### 11.2 待确认的技术细节

1. 是否需要支持 `parking_lot` 的锁类型？
2. 是否需要提供无锁版本（仅用于基准测试对比）？
3. 错误消息的国际化（i18n）支持？
4. 是否需要提供宏来简化常见用法？

## 12. 参考资料

- [Java 版本源码](../external/common-java/src/main/java/ltd/qubit/commons/concurrent/DoubleCheckedLockExecutor.java)
- [Rust 并发模型](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rust Atomics and Locks](https://marabos.nl/atomics/)
- [prism3-rust-function 文档](../prism3-rust-function/README.md)
- [prism3-rust-clock 文档](../prism3-rust-clock/README.md)

## 13. 变更历史

| 版本 | 日期 | 作者 | 变更说明 |
|-----|------|------|---------|
| 1.0 | 2025-01-XX | AI Assistant | 初始版本 |

---

**文档状态**：草案
**最后更新**：2025-01-XX
**审阅者**：待定

