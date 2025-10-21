# Rust `CasExecutor` 移植设计方案

这是一个将Java实现的 `CasExecutor` 组件移植到Rust的详细设计方案。方案旨在完整保留原组件功能的同时，充分利用Rust语言的特性来提升代码的安全性、性能和易用性。

### 1. Java `CasExecutor` 核心功能分析

原始的Java `CasExecutor` 是一个通用的乐观锁工具，其核心特性包括：

1.  **核心逻辑**：在一个CAS循环中执行用户提供的操作（`LoopBody`），在数据竞争时自动重试。
2.  **通用性与泛型**：通过泛型 `<T, R>` 支持任意类型的受保护数据和返回结果。
3.  **灵活的配置**：通过 `Options` 类支持配置最大重试次数、超时时间和重试延迟策略。
4.  **同步与异步执行**：同时提供 `execute` (同步) 和 `executeAsync` (异步) 方法。
5.  **精细的循环控制**：通过一个可变的 `CasResult` 对象，允许 `LoopBody` 控制循环是成功更新、成功不更新、重试还是中止。

### 2. Rust 移植设计方案

目标是创建一个功能对等、符合Rust语言习惯且绝对线程安全的 `CasExecutor`。

#### 2.1. 整体架构设计

我们将使用更符合Rust风格的结构和类型来重新组织这个组件。

```mermaid
graph TD
    subgraph Rust 实现: cas_executor
        direction LR

        A[CasExecutor Struct] -- 拥有 --> B(CasOptions Struct);
        A -- "执行操作(execute, execute_async)" --> C{"FnMut(Arc&lt;T&gt;) —> Result&lt;CasAction&lt;T, R&gt;, E&gt; (闭包)"};
        C -- 返回 --> D[CasAction Enum];
        D -- "包含" --> E[用户定义的结果类型 R];
        D -- "包含" --> F[CAS保护的状态类型 T];
        A -- 操作于 --> G["Arc&lt;ArcSwap&lt;T&gt;&gt; (原子状态)"];
        A -- "返回" -> H["Result&lt;R, CasError&lt;E&gt;&gt;"];
    end

    subgraph Java 原版
        direction LR
        J[CasExecutor Class] -- 拥有 --> K(Options Class);
        J -- "执行操作(execute, executeAsync)" --> L[LoopBody Interface];
        L -- "修改" --> M[CasResult Class];
        M -- "包含" --> N[用户定义的结果类型 R];
        M -- "包含" --> O[CAS保护的状态类型 T];
        J -- 操作于 --> P[AtomicReference&lt;T&gt;];
        J -- "返回" --> N;
    end

    style A fill:#3E7,stroke:#333,stroke-width:2px
    style G fill:#E77,stroke:#333,stroke-width:2px
    style C fill:#E7E,stroke:#333,stroke-width:2px
    style H fill:#3E7,stroke:#333,stroke-width:2px

    style J fill:#7CF,stroke:#333,stroke-width:2px
    style P fill:#E77,stroke:#333,stroke-width:2px
    style L fill:#EEA,stroke:#333,stroke-width:2px
```

**关键映射关系**：

*   **`CasExecutor` 结构体**: 代替Java的 `CasExecutor` 类，并持有配置。
*   **`CasOptions` 结构体**: 代替Java的 `Options` 类，并提供 **Builder模式**。
*   **原子状态管理**: 使用 `arc-swap` crate 中的 `ArcSwap<T>` 代替Java的 `AtomicReference<T>`，以实现对 `Arc<T>` 的高效原子操作。
*   **循环体 (`LoopBody`)**: 使用Rust的 **闭包** 代替Java的 `LoopBody` 接口，使API更灵活、更函数式。
*   **循环控制 (`CasResult`)**: 使用一个返回的 **枚举 `CasAction<T, R>`** 来代替修改外部 `CasResult` 对象的方式，使意图更明确。

#### 2.2. 关键类型定义 (Rust-style)

1.  **`CasOptions` 和构建器**:
    ```rust
    #[derive(Clone, Debug)]
    pub struct CasOptions {
        pub max_retries: u32,
        pub timeout: Duration,
        pub delay_strategy: DelayStrategy,
    }

    // 将提供一个 CasOptions::builder() 方法来链式构建配置
    ```

2.  **`DelayStrategy` 延迟策略**:
    ```rust
    #[derive(Clone, Debug)]
    pub enum DelayStrategy {
        Fixed(Duration),
        ExponentialBackoff {
            initial_delay: Duration,
            max_delay: Duration,
            factor: f64,
        },
        // ... 其他策略
    }
    ```

3.  **`CasAction` 枚举（替代 `CasResult`）**:
    ```rust
    pub enum CasAction<T, R> {
        /// 尝试使用 `new_value` 更新状态。
        /// 如果CAS成功，整个操作将成功并返回 `result`。
        Update { new_value: T, result: R },

        /// 不更新状态，但立即成功完成操作，并返回 `result`。
        Finish { result: R },

        /// 不更新状态，并要求 `CasExecutor` 立即重试循环。
        Retry,
    }
    ```

4.  **`CasError` 错误类型** (使用 `thiserror`):
    ```rust
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum CasError<E> {
        #[error("操作在 {0:?} 后超时")]
        Timeout(Duration),

        #[error("超过最大重试次数: {0}")]
        MaxRetriesExceeded(u32),

        #[error("循环体执行失败")]
        LoopBodyFailed(#[from] E), // 封装用户闭包的错误
    }
    ```

#### 2.3. `CasExecutor` 的 API 设计

```rust
use arc_swap::ArcSwap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct CasExecutor {
    options: CasOptions,
}

impl CasExecutor {
    pub fn new(options: CasOptions) -> Self {
        Self { options }
    }

    /// 同步执行CAS循环
    pub fn execute<T, F, R, E>(
        &self,
        target: &Arc<ArcSwap<T>>,
        body: F,
    ) -> Result<R, CasError<E>>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
        E: Send + Sync + 'static,
        F: FnMut(Arc<T>) -> Result<CasAction<T, R>, E>,
    {
        // ... 具体的循环、重试、超时和延迟逻辑 ...
        todo!()
    }

    /// 异步执行CAS循环
    pub async fn execute_async<T, F, Fut, R, E>(
        &self,
        target: &Arc<ArcSwap<T>>,
        body: F,
    ) -> Result<R, CasError<E>>
    where
        T: Send + Sync + 'static,
        R: Send + 'static,
        E: Send + Sync + 'static,
        F: FnMut(Arc<T>) -> Fut,
        Fut: std::future::Future<Output = Result<CasAction<T, R>, E>> + Send,
    {
        // ... 异步版本的逻辑 ...
        todo!()
    }
}
```

### 3. 方案优势总结

*   **类型安全**: Rust的所有权模型、`CasAction`和`CasError`枚举从根本上杜绝了数据竞争和不明确的状态转换。
*   **性能**: `arc-swap` crate 提供了比锁更高效的原子读操作，性能有望超越Java版本。
*   **人体工程学**: Builder模式、闭包和`Result`类型的使用使API更符合现代Rust风格，易于使用且不易出错。
*   **现代并发模型**: `async/await`的无缝集成，使其能轻松融入现代Rust异步生态。

这个方案为移植工作提供了一个清晰、健壮且符合Rust语言哲学的蓝图。
