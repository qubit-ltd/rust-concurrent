# DoubleCheckedLockExecutor 使用 ArcTester 的设计说明

## 概述

`DoubleCheckedLockExecutor` 在构建阶段通过 `when` 接收任何实现
`qubit_function::Tester` 的条件测试器，并统一保存为 `ArcTester`。
这样 executor 可以预先配置并多次复用，每次调用 `call`、`execute`、
`call_with` 或 `execute_with` 时都会执行同一套双重检查流程。

条件会被执行两次：

1. 加锁前执行一次，用于快速跳过不需要进入临界区的任务。
2. 获取写锁后再执行一次，用于避免两次检查之间状态发生变化。

因此，`when` 闭包在锁外读取的状态必须本身是并发安全的，例如使用
`AtomicBool`、其他锁保护的状态，或只读不可变状态。

## 使用闭包

闭包会自动实现 `Tester` trait：

```rust
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use qubit_concurrent::{DoubleCheckedLockExecutor, lock::ArcMutex};

let running = Arc::new(AtomicBool::new(true));
let data = ArcMutex::new(42);

let executor = DoubleCheckedLockExecutor::builder()
    .on(data.clone())
    .when({
        let running = running.clone();
        move || running.load(Ordering::Acquire)
    })
    .build();

let result = executor
    .call_with(|value: &mut i32| {
        *value += 1;
        Ok::<_, std::io::Error>(*value)
    })
    .get_result();
```

## 使用 BoxTester

`BoxTester` 适合在构建阶段动态组装条件：

```rust
use qubit_function::BoxTester;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use qubit_concurrent::{DoubleCheckedLockExecutor, lock::ArcMutex};

let running = Arc::new(AtomicBool::new(true));
let data = ArcMutex::new(42);

let tester = {
    let running = running.clone();
    BoxTester::new(move || running.load(Ordering::Acquire))
};

let executor = DoubleCheckedLockExecutor::builder()
    .on(data.clone())
    .when(tester)
    .build();
```

## 使用 ArcTester

`ArcTester` 适合需要克隆并跨线程共享同一个条件的场景：

```rust
use qubit_function::ArcTester;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use qubit_concurrent::{DoubleCheckedLockExecutor, lock::ArcMutex};

let running = Arc::new(AtomicBool::new(true));
let data = ArcMutex::new(42);

let tester = {
    let running = running.clone();
    ArcTester::new(move || running.load(Ordering::Acquire))
};

let executor = DoubleCheckedLockExecutor::builder()
    .on(data.clone())
    .when(tester.clone())
    .build();
```

## 组合多个 Tester

可以使用 `qubit-function` 提供的逻辑组合方法组合条件：

```rust
use qubit_function::BoxTester;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use qubit_concurrent::{DoubleCheckedLockExecutor, lock::ArcMutex};

let running = Arc::new(AtomicBool::new(true));
let ready = Arc::new(AtomicBool::new(true));
let data = ArcMutex::new(42);

let running_tester = {
    let running = running.clone();
    BoxTester::new(move || running.load(Ordering::Acquire))
};

let ready_tester = {
    let ready = ready.clone();
    move || ready.load(Ordering::Acquire)
};

let executor = DoubleCheckedLockExecutor::builder()
    .on(data.clone())
    .when(running_tester.and(ready_tester))
    .build();
```

## 与任务接口的关系

`when` 只负责条件判断。真正的业务任务在 executor 构建后提交：

- `call(Callable)`：执行零参数、有返回值的任务。
- `execute(Runnable)`：执行零参数、无返回值的任务。
- `call_with(CallableWith)`：执行接收 `&mut T` 且有返回值的任务。
- `execute_with(RunnableWith)`：执行接收 `&mut T` 且无返回值的任务。

其中 `Callable`、`Runnable`、`CallableWith` 和 `RunnableWith` 都来自
`qubit-function`，并支持 `FnMut` 风格的可复用任务。

## 示例

运行基本示例：

```bash
cargo run --example double_checked_lock_executor_demo
```

运行测试：

```bash
cargo test double_checked
```
