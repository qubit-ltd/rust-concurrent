# DoubleCheckedLockExecutor 设计说明

本文档描述 `qubit-concurrent` 中 `DoubleCheckedLockExecutor` 的当前设计。
它借鉴 Java 版 `DoubleCheckedLockExecutor` 的“先配置、后多次执行”模型，
但实现上遵循 Rust 的所有权、trait 和闭包语义。

## 目标

- executor 可以预先配置锁、条件判断、prepare / rollback / commit 生命周期钩子。
- executor 构建完成后可以多次调用 `call`、`execute`、`call_with`、`execute_with`。
- 任务接口复用 `qubit-function` 的 `Callable`、`Runnable`、`CallableWith`、`RunnableWith`。
- 锁访问继续使用 `Lock<T>` 的闭包式 API，避免把锁守卫暴露给调用方。
- 条件判断保留双重检查语义：锁外先判断一次，持写锁后再判断一次。

## API 形态

```rust
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use qubit_concurrent::{DoubleCheckedLockExecutor, ArcMutex};

let account_balance = ArcMutex::new(1_000);
let transaction_active = Arc::new(AtomicBool::new(false));

let executor = DoubleCheckedLockExecutor::builder()
    .on(account_balance.clone())
    .when({
        let transaction_active = transaction_active.clone();
        move || !transaction_active.load(Ordering::Acquire)
    })
    .prepare(|| Ok::<(), std::io::Error>(()))
    .rollback_prepare(|| Ok::<(), std::io::Error>(()))
    .commit_prepare(|| Ok::<(), std::io::Error>(()))
    .build();

let result = executor.call_with({
    let transaction_active = transaction_active.clone();
    move |balance: &mut i32| {
        transaction_active.store(true, Ordering::Release);
        *balance -= 100;
        Ok::<_, std::io::Error>(*balance)
    }
});

executor.execute_with(|balance: &mut i32| {
    println!("balance = {balance}");
    Ok::<(), std::io::Error>(())
});
```

## 执行流程

每次执行任务时，executor 都按同一流程运行：

1. 执行 `when` 条件。若返回 `false`，直接返回 `ExecutionResult::ConditionNotMet`。
2. 若配置了 `prepare`，执行 prepare。prepare 失败时返回 `PrepareFailed`。
3. 获取写锁。
4. 在写锁内再次执行 `when` 条件。若返回 `false`，返回 `ConditionNotMet`。
5. 执行业务任务。
6. 释放写锁。
7. 如果 prepare 已成功执行：
   - 任务成功时执行 `commit_prepare`。
   - 条件未满足或任务失败时执行 `rollback_prepare`。

prepare 的提交或回滚在写锁释放后执行，避免把生命周期清理动作放大到临界区内。

## 任务接口

`DoubleCheckedLockExecutor` 提供四个任务入口：

| 方法 | 接收任务 | 是否接收受保护数据 | 返回 |
|------|----------|--------------------|------|
| `call` | `Callable<R, E>` | 否 | `ExecutionContext<R, E>` |
| `execute` | `Runnable<E>` | 否 | `ExecutionContext<(), E>` |
| `call_with` | `CallableWith<T, R, E>` | 是，`&mut T` | `ExecutionContext<R, E>` |
| `execute_with` | `RunnableWith<T, E>` | 是，`&mut T` | `ExecutionContext<(), E>` |

`call` 和 `execute` 同时也是 `Executor` 风格的 API，适合任务本身不需要直接访问锁内数据、
但仍需要复用同一套双重检查和 prepare 生命周期的场景。

## 锁所有权

builder 的 `.on(lock)` 接收锁句柄本身，而不是借用。`ArcMutex<T>` 和 `ArcRwLock<T>`
内部已经集成 `Arc`，因此推荐传入 `.clone()` 得到的轻量句柄：

```rust
let data = ArcMutex::new(0);

let executor = DoubleCheckedLockExecutor::builder()
    .on(data.clone())
    .when(|| true)
    .build();
```

这样 executor 可以长期持有锁句柄并多次执行任务。

## 条件测试器

`when` 接收任何实现 `qubit_function::Tester` 的类型，并在内部保存为 `ArcTester`。
闭包、`BoxTester`、`ArcTester` 以及组合后的 tester 都可以直接使用。

由于第一次条件判断发生在锁外，条件读取的状态必须本身是并发安全的。例如：

- `AtomicBool` / `AtomicUsize` 等原子变量。
- 其他锁保护的数据。
- 初始化后不再变化的不可变数据。

## 错误模型

业务任务错误保留原始 `E`，并包装为：

- `ExecutionResult::Success(T)`
- `ExecutionResult::ConditionNotMet`
- `ExecutionResult::Failed(ExecutorError<E>)`

prepare / commit / rollback 的错误会转成字符串，因为这些生命周期钩子可以与业务任务使用不同错误类型。
这避免了把多个不相关的错误类型强行合并到同一个泛型参数中。

## 模块结构

核心代码位于：

- `src/double_checked/double_checked_lock_executor.rs`
- `src/double_checked/execution_context.rs`
- `src/double_checked/execution_result.rs`
- `src/double_checked/executor_error.rs`
- `src/double_checked/execution_logger.rs`

测试位于：

- `tests/double_checked/double_checked_lock_executor_tests.rs`

示例位于：

- `examples/double_checked_lock_executor_demo.rs`
