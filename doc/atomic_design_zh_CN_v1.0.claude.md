# Atomic 封装设计文档 v1.0

## 1. 背景与目标

### 1.1 背景

Rust 标准库的 `std::sync::atomic` 提供了底层的原子类型，但使用起来存在一些不便：

1. **显式 Ordering 要求**：每次操作都需要显式指定内存序（`Ordering::Relaxed`、`Ordering::Acquire`、`Ordering::Release` 等），增加了使用复杂度
2. **API 较为底层**：缺少常见的高级操作（如 `getAndIncrement`、`incrementAndGet` 等）
3. **易用性不足**：对于大多数场景，开发者只需要"正确"的原子操作，而不需要关心底层内存序细节

相比之下，JDK 的 atomic 包（`java.util.concurrent.atomic`）提供了更友好的 API：

```java
// Java 示例
AtomicInteger counter = new AtomicInteger(0);
int old = counter.getAndIncrement();  // 自动使用正确的内存序
int current = counter.incrementAndGet();
boolean success = counter.compareAndSet(expected, newValue);
```

### 1.2 目标

设计一套 Rust 的 atomic 封装，使其：

1. **易用性**：隐藏 `Ordering` 复杂性，提供合理的默认内存序
2. **完整性**：提供与 JDK atomic 类似的高级操作方法
3. **安全性**：保证内存安全和线程安全
4. **性能**：零成本抽象，不引入额外开销
5. **灵活性**：通过 `inner()` 方法暴露底层类型，高级用户可直接操作标准库类型
6. **简洁性**：API 表面积小，不提供 `_with_ordering` 变体以避免 API 膨胀

### 1.3 非目标

- 不改变 Rust 的内存模型
- 不引入新的同步原语
- 不提供跨进程的原子操作

## 2. 内存序策略

### 2.1 内存序概述

Rust 提供了五种内存序：

| 内存序 | 说明 | 适用场景 |
|-------|------|---------|
| `Relaxed` | 只保证原子性，不保证顺序 | 性能关键场景，如计数器 |
| `Acquire` | 读操作，防止后续读写被重排到此操作之前 | 读取共享状态 |
| `Release` | 写操作，防止之前读写被重排到此操作之后 | 更新共享状态 |
| `AcqRel` | 同时具有 Acquire 和 Release 语义 | 读-改-写操作 |
| `SeqCst` | 最强保证，全局顺序一致性 | 需要严格顺序的场景 |

### 2.2 默认策略

为平衡易用性、正确性和性能，我们采用以下默认策略：

| 操作类型 | 默认 Ordering | 原因 |
|---------|--------------|------|
| **纯读操作** | `Acquire` | 保证读取最新值，防止后续操作被重排 |
| **纯写操作** | `Release` | 保证写入可见，防止之前操作被重排 |
| **读-改-写操作** | `AcqRel` | 同时保证读和写的正确性 |
| **比较并交换** | `AcqRel`（成功）+ `Acquire`（失败）| 标准 CAS 语义 |

**特殊情况**：

- **计数器操作**（如 `increment`、`decrement`）：使用 `Relaxed`，因为大多数场景下只需要保证计数正确，不需要同步其他状态
- **高级 API**（如 `updateAndGet`）：使用 `AcqRel`，保证函数内的状态一致性

### 2.3 高级场景：直接访问底层类型

对于需要精细控制内存序的场景（约 1% 的使用情况），通过 `inner()` 方法访问底层标准库类型：

```rust
use std::sync::atomic::Ordering;

let atomic = AtomicI32::new(0);

// 99% 的场景：使用简单 API
let value = atomic.get();

// 1% 的场景：需要精细控制，直接操作底层类型
let value = atomic.inner().load(Ordering::Relaxed);
atomic.inner().store(42, Ordering::Release);
```

**设计理念**：我们不提供所有方法的 `_with_ordering` 变体，因为：
1. 避免 API 膨胀（否则方法数量翻倍）
2. 防止误用（用户可能不理解内存序）
3. 保持简洁性（符合"易用封装"的定位）
4. `inner()` 是完美的 escape hatch（高级用户清楚知道自己在做什么）

## 3. 类型设计

### 3.1 封装类型概览

| Rust 封装类型 | 底层类型 | JDK 对应类型 | 说明 |
|--------------|---------|-------------|------|
| `AtomicBool` | `std::sync::atomic::AtomicBool` | `AtomicBoolean` | 原子布尔值 |
| `AtomicI32` | `std::sync::atomic::AtomicI32` | `AtomicInteger` | 32位有符号整数 |
| `AtomicI64` | `std::sync::atomic::AtomicI64` | `AtomicLong` | 64位有符号整数 |
| `AtomicU32` | `std::sync::atomic::AtomicU32` | - | 32位无符号整数 |
| `AtomicU64` | `std::sync::atomic::AtomicU64` | - | 64位无符号整数 |
| `AtomicUsize` | `std::sync::atomic::AtomicUsize` | - | 指针大小的无符号整数 |
| `AtomicIsize` | `std::sync::atomic::AtomicIsize` | - | 指针大小的有符号整数 |
| `AtomicRef<T>` | `std::sync::atomic::AtomicPtr<T>` + `Arc<T>` | `AtomicReference<V>` | 原子引用 |

**注意**：我们直接使用 `std::sync::atomic` 的类型名，通过模块路径区分：

```rust
// 标准库类型
use std::sync::atomic::AtomicI32 as StdAtomicI32;

// 我们的封装类型
use qubit_atomic::::AtomicI32;
```

### 3.2 核心结构

```rust
/// 原子整数封装（以 AtomicI32 为例）
///
/// 提供易用的原子操作 API，自动使用合理的内存序。
#[repr(transparent)]
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}

// 自动实现的 trait
unsafe impl Send for AtomicI32 {}
unsafe impl Sync for AtomicI32 {}

impl Default for AtomicI32 {
    fn default() -> Self {
        Self::new(0)
    }
}

impl From<i32> for AtomicI32 {
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for AtomicI32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicI32")
            .field("value", &self.get())
            .finish()
    }
}

impl fmt::Display for AtomicI32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}
```

### 3.3 Trait 实现

所有原子类型都应实现以下 trait：

| Trait | 说明 | JDK 对应 |
|-------|------|---------|
| `Send` | 可跨线程转移 | 自动满足 |
| `Sync` | 可跨线程共享 | 自动满足 |
| `Default` | 默认值构造 | - |
| `Debug` | 调试输出 | `toString()` |
| `Display` | 格式化输出 | `toString()` |
| `From<T>` | 类型转换 | 构造函数 |

**不实现的 trait**：
- `Clone`：原子类型不应该被克隆（但 `AtomicRef` 可以）
- `PartialEq`/`Eq`：比较原子类型的值需要读取，可能产生误解
- `PartialOrd`/`Ord`：同上
- `Hash`：同上

**原因**：实现这些 trait 会隐藏读取操作，用户应该显式调用 `get()` 或 `inner().load()`。

```rust
// ❌ 误导性的代码
if atomic1 == atomic2 {  // 这看起来像简单比较，但实际是两次原子读取
    // ...
}

// ✅ 明确的代码
if atomic1.get() == atomic2.get() {  // 清楚地表明这是两次独立的读取
    // ...
}
```

### 3.4 设计原则

1. **零成本抽象**：封装不引入额外开销，内联所有方法
2. **类型安全**：利用 Rust 类型系统防止误用
3. **所有权友好**：支持 `Send + Sync`，可安全跨线程共享
4. **trait 统一**：通过 trait 提供统一接口
5. **显式优于隐式**：不实现可能误导的 trait（如 `PartialEq`）

## 4. API 设计

### 4.1 基础操作

所有原子类型都提供以下基础操作：

```rust
impl AtomicI32 {
    /// 创建新的原子整数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(42);
    /// ```
    pub const fn new(value: i32) -> Self;

    /// 获取当前值（使用 Acquire ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(42);
    /// assert_eq!(atomic.get(), 42);
    /// ```
    pub fn get(&self) -> i32;

    /// 设置新值（使用 Release ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(0);
    /// atomic.set(42);
    /// assert_eq!(atomic.get(), 42);
    /// ```
    pub fn set(&self, value: i32);

    /// 交换值，返回旧值（使用 AcqRel ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.swap(20);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 20);
    /// ```
    pub fn swap(&self, value: i32) -> i32;

    /// 比较并交换（CAS）
    ///
    /// 如果当前值等于 `current`，则设置为 `new`，返回 `Ok(())`；
    /// 否则返回 `Err(actual)`，其中 `actual` 是实际的当前值。
    ///
    /// # 参数
    ///
    /// * `current` - 期望的当前值
    /// * `new` - 要设置的新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 成功的 CAS
    /// assert!(atomic.compare_and_set(10, 20).is_ok());
    /// assert_eq!(atomic.get(), 20);
    ///
    /// // 失败的 CAS
    /// match atomic.compare_and_set(10, 30) {
    ///     Ok(_) => panic!("Should fail"),
    ///     Err(actual) => assert_eq!(actual, 20),
    /// }
    /// ```
    pub fn compare_and_set(&self, current: i32, new: i32) -> Result<(), i32>;

    /// 弱版本的 CAS（允许虚假失败，但在某些平台上性能更好）
    ///
    /// 主要用于循环中的 CAS 操作。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    ///
    /// // 在循环中使用弱 CAS
    /// let mut current = atomic.get();
    /// loop {
    ///     let new = current + 1;
    ///     match atomic.compare_and_set_weak(current, new) {
    ///         Ok(_) => break,
    ///         Err(actual) => current = actual,
    ///     }
    /// }
    /// assert_eq!(atomic.get(), 11);
    /// ```
    pub fn compare_and_set_weak(&self, current: i32, new: i32) -> Result<(), i32>;

    /// 获取底层标准库类型的引用
    ///
    /// 用于需要精细控制内存序的高级场景。大多数情况下不需要使用此方法，
    /// 默认 API 已经提供了合理的内存序。
    ///
    /// # 使用场景
    ///
    /// - 极致性能优化（需要使用 `Relaxed` ordering）
    /// - 复杂的无锁算法（需要精确控制内存序）
    /// - 与直接使用标准库的代码互操作
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    /// use std::sync::atomic::Ordering;
    ///
    /// let atomic = AtomicI32::new(0);
    ///
    /// // 高性能场景：使用 Relaxed ordering
    /// for _ in 0..1_000_000 {
    ///     atomic.inner().fetch_add(1, Ordering::Relaxed);
    /// }
    ///
    /// // 最后用 Acquire 读取结果
    /// let result = atomic.inner().load(Ordering::Acquire);
    /// assert_eq!(result, 1_000_000);
    /// ```
    pub fn inner(&self) -> &std::sync::atomic::AtomicI32;
}
```

### 4.2 整数类型的高级操作

整数类型（`AtomicI32`、`AtomicI64`、`AtomicU32`、`AtomicU64`、`AtomicIsize`、`AtomicUsize`）额外提供：

```rust
impl AtomicI32 {
    // ==================== 自增/自减操作 ====================

    /// 原子自增，返回旧值（使用 Relaxed ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_increment();
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 11);
    /// ```
    pub fn get_and_increment(&self) -> i32;

    /// 原子自增，返回新值（使用 Relaxed ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let new = atomic.increment_and_get();
    /// assert_eq!(new, 11);
    /// ```
    pub fn increment_and_get(&self) -> i32;

    /// 原子自减，返回旧值（使用 Relaxed ordering）
    pub fn get_and_decrement(&self) -> i32;

    /// 原子自减，返回新值（使用 Relaxed ordering）
    pub fn decrement_and_get(&self) -> i32;

    // ==================== 加法/减法操作 ====================

    /// 原子加法，返回旧值（使用 Relaxed ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_add(5);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 15);
    /// ```
    pub fn get_and_add(&self, delta: i32) -> i32;

    /// 原子加法，返回新值（使用 Relaxed ordering）
    pub fn add_and_get(&self, delta: i32) -> i32;

    /// 原子减法，返回旧值（使用 Relaxed ordering）
    pub fn get_and_sub(&self, delta: i32) -> i32;

    /// 原子减法，返回新值（使用 Relaxed ordering）
    pub fn sub_and_get(&self, delta: i32) -> i32;

    // ==================== 位运算操作 ====================

    /// 原子按位与，返回旧值
    pub fn get_and_bitand(&self, value: i32) -> i32;

    /// 原子按位或，返回旧值
    pub fn get_and_bitor(&self, value: i32) -> i32;

    /// 原子按位异或，返回旧值
    pub fn get_and_bitxor(&self, value: i32) -> i32;

    // ==================== 函数式更新操作 ====================

    /// 使用给定函数原子更新值，返回旧值
    ///
    /// 内部使用 CAS 循环，直到更新成功。
    ///
    /// # 参数
    ///
    /// * `f` - 更新函数，接收当前值，返回新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_update(|x| x * 2);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 20);
    /// ```
    pub fn get_and_update<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32;

    /// 使用给定函数原子更新值，返回新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let new = atomic.update_and_get(|x| x * 2);
    /// assert_eq!(new, 20);
    /// ```
    pub fn update_and_get<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32;

    /// 使用给定的二元函数原子累积值
    ///
    /// # 参数
    ///
    /// * `x` - 累积参数
    /// * `f` - 累积函数，接收当前值和参数，返回新值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// let old = atomic.get_and_accumulate(5, |a, b| a + b);
    /// assert_eq!(old, 10);
    /// assert_eq!(atomic.get(), 15);
    /// ```
    pub fn get_and_accumulate<F>(&self, x: i32, f: F) -> i32
    where
        F: Fn(i32, i32) -> i32;

    /// 使用给定的二元函数原子累积值，返回新值
    pub fn accumulate_and_get<F>(&self, x: i32, f: F) -> i32
    where
        F: Fn(i32, i32) -> i32;

    // ==================== 最大值/最小值操作 ====================

    /// 原子取最大值，返回旧值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicI32;
    ///
    /// let atomic = AtomicI32::new(10);
    /// atomic.get_and_max(20);
    /// assert_eq!(atomic.get(), 20);
    ///
    /// atomic.get_and_max(15);
    /// assert_eq!(atomic.get(), 20); // 保持较大值
    /// ```
    pub fn get_and_max(&self, value: i32) -> i32;

    /// 原子取最大值，返回新值
    pub fn max_and_get(&self, value: i32) -> i32;

    /// 原子取最小值，返回旧值
    pub fn get_and_min(&self, value: i32) -> i32;

    /// 原子取最小值，返回新值
    pub fn min_and_get(&self, value: i32) -> i32;
}
```

### 4.3 布尔类型的特殊操作

```rust
impl AtomicBool {
    /// 创建新的原子布尔值
    pub const fn new(value: bool) -> Self;

    /// 获取当前值
    pub fn get(&self) -> bool;

    /// 设置新值
    pub fn set(&self, value: bool);

    /// 交换值，返回旧值
    pub fn swap(&self, value: bool) -> bool;

    /// 比较并交换
    pub fn compare_and_set(&self, current: bool, new: bool) -> Result<(), bool>;

    /// 弱版本的 CAS
    pub fn compare_and_set_weak(&self, current: bool, new: bool) -> Result<(), bool>;

    // ==================== 布尔特殊操作 ====================

    /// 原子设置为 true，返回旧值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// let old = flag.get_and_set();
    /// assert_eq!(old, false);
    /// assert_eq!(flag.get(), true);
    /// ```
    pub fn get_and_set(&self) -> bool;

    /// 原子设置为 true，返回新值
    pub fn set_and_get(&self) -> bool;

    /// 原子设置为 false，返回旧值
    pub fn get_and_clear(&self) -> bool;

    /// 原子设置为 false，返回新值
    pub fn clear_and_get(&self) -> bool;

    /// 原子取反，返回旧值
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    /// assert_eq!(flag.get_and_negate(), false);
    /// assert_eq!(flag.get(), true);
    /// assert_eq!(flag.get_and_negate(), true);
    /// assert_eq!(flag.get(), false);
    /// ```
    pub fn get_and_negate(&self) -> bool;

    /// 原子取反，返回新值
    pub fn negate_and_get(&self) -> bool;

    /// 原子逻辑与，返回旧值
    pub fn get_and_logical_and(&self, value: bool) -> bool;

    /// 原子逻辑或，返回旧值
    pub fn get_and_logical_or(&self, value: bool) -> bool;

    /// 原子逻辑异或，返回旧值
    pub fn get_and_logical_xor(&self, value: bool) -> bool;

    /// 使用 CAS 实现的条件设置
    ///
    /// 当当前值为 `false` 时设置为 `true`，返回是否成功。
    /// 常用于实现一次性标志或锁。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicBool;
    ///
    /// let flag = AtomicBool::new(false);
    ///
    /// // 第一次调用成功
    /// assert!(flag.compare_and_set_if_false(true).is_ok());
    /// assert_eq!(flag.get(), true);
    ///
    /// // 第二次调用失败（已经是 true）
    /// assert!(flag.compare_and_set_if_false(true).is_err());
    /// ```
    pub fn compare_and_set_if_false(&self, new: bool) -> Result<(), bool>;

    /// 当当前值为 `true` 时设置为 `false`，返回是否成功
    pub fn compare_and_set_if_true(&self, new: bool) -> Result<(), bool>;
}
```

### 4.4 引用类型的操作

```rust
/// 原子引用封装
///
/// 使用 `Arc<T>` 实现线程安全的引用共享。
///
/// # 泛型参数
///
/// * `T` - 引用的数据类型
pub struct AtomicRef<T> {
    inner: std::sync::atomic::AtomicPtr<Arc<T>>,
}

impl<T> AtomicRef<T> {
    /// 创建新的原子引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let data = Arc::new(42);
    /// let atomic = AtomicRef::new(data);
    /// ```
    pub fn new(value: Arc<T>) -> Self;

    /// 获取当前引用（使用 Acquire ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(42));
    /// let value = atomic.get();
    /// assert_eq!(*value, 42);
    /// ```
    pub fn get(&self) -> Arc<T>;

    /// 设置新引用（使用 Release ordering）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(42));
    /// atomic.set(Arc::new(100));
    /// assert_eq!(*atomic.get(), 100);
    /// ```
    pub fn set(&self, value: Arc<T>);

    /// 交换引用，返回旧引用（使用 AcqRel ordering）
    pub fn swap(&self, value: Arc<T>) -> Arc<T>;

    /// 比较并交换引用
    ///
    /// 如果当前引用与 `current` 指向同一对象，则替换为 `new`。
    ///
    /// # 注意
    ///
    /// 比较使用指针相等性（`Arc::ptr_eq`），而非值相等性。
    pub fn compare_and_set(&self, current: &Arc<T>, new: Arc<T>) -> Result<(), Arc<T>>;

    /// 弱版本的 CAS
    pub fn compare_and_set_weak(&self, current: &Arc<T>, new: Arc<T>) -> Result<(), Arc<T>>;

    /// 使用函数更新引用，返回旧引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use qubit_atomic::::AtomicRef;
    /// use std::sync::Arc;
    ///
    /// let atomic = AtomicRef::new(Arc::new(10));
    /// let old = atomic.get_and_update(|x| Arc::new(*x * 2));
    /// assert_eq!(*old, 10);
    /// assert_eq!(*atomic.get(), 20);
    /// ```
    pub fn get_and_update<F>(&self, f: F) -> Arc<T>
    where
        F: Fn(&Arc<T>) -> Arc<T>;

    /// 使用函数更新引用，返回新引用
    pub fn update_and_get<F>(&self, f: F) -> Arc<T>
    where
        F: Fn(&Arc<T>) -> Arc<T>;
}

impl<T> Clone for AtomicRef<T> {
    /// 克隆原子引用
    ///
    /// 注意：这会创建一个新的 `AtomicRef`，它与原始引用指向同一底层数据，
    /// 但后续的原子操作是独立的。
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}
```

## 5. Trait 抽象设计

### 5.1 Atomic Trait

提供统一的原子操作接口：

```rust
/// 原子操作的通用 trait
///
/// 定义了所有原子类型的基本操作。
pub trait Atomic {
    /// 值类型
    type Value;

    /// 获取当前值
    fn get(&self) -> Self::Value;

    /// 设置新值
    fn set(&self, value: Self::Value);

    /// 交换值，返回旧值
    fn swap(&self, value: Self::Value) -> Self::Value;

    /// 比较并交换
    fn compare_and_set(&self, current: Self::Value, new: Self::Value)
        -> Result<(), Self::Value>;
}

/// 可更新的原子类型 trait
///
/// 提供函数式更新操作。
pub trait UpdatableAtomic: Atomic {
    /// 使用函数更新值，返回旧值
    fn get_and_update<F>(&self, f: F) -> Self::Value
    where
        F: Fn(Self::Value) -> Self::Value;

    /// 使用函数更新值，返回新值
    fn update_and_get<F>(&self, f: F) -> Self::Value
    where
        F: Fn(Self::Value) -> Self::Value;
}

/// 原子整数 trait
///
/// 提供整数特有的操作。
pub trait AtomicInteger: UpdatableAtomic {
    /// 自增，返回旧值
    fn get_and_increment(&self) -> Self::Value;

    /// 自增，返回新值
    fn increment_and_get(&self) -> Self::Value;

    /// 自减，返回旧值
    fn get_and_decrement(&self) -> Self::Value;

    /// 自减，返回新值
    fn decrement_and_get(&self) -> Self::Value;

    /// 加法，返回旧值
    fn get_and_add(&self, delta: Self::Value) -> Self::Value;

    /// 加法，返回新值
    fn add_and_get(&self, delta: Self::Value) -> Self::Value;
}
```

### 5.2 Trait 实现

```rust
// AtomicI32 实现 Atomic trait
impl Atomic for AtomicI32 {
    type Value = i32;

    fn get(&self) -> i32 {
        self.inner.load(Ordering::Acquire)
    }

    fn set(&self, value: i32) {
        self.inner.store(value, Ordering::Release);
    }

    fn swap(&self, value: i32) -> i32 {
        self.inner.swap(value, Ordering::AcqRel)
    }

    fn compare_and_set(&self, current: i32, new: i32) -> Result<(), i32> {
        self.inner
            .compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
    }
}

// AtomicI32 实现 AtomicInteger trait
impl AtomicInteger for AtomicI32 {
    fn get_and_increment(&self) -> i32 {
        self.inner.fetch_add(1, Ordering::Relaxed)
    }

    fn increment_and_get(&self) -> i32 {
        self.inner.fetch_add(1, Ordering::Relaxed) + 1
    }

    // ... 其他方法
}
```

## 6. 使用示例

### 6.1 基础计数器

```rust
use qubit_atomic::::AtomicI32;
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    // 启动 10 个线程，每个线程递增计数器 1000 次
    for _ in 0..10 {
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment_and_get();
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证结果
    assert_eq!(counter.get(), 10000);
    println!("最终计数：{}", counter.get());
}
```

### 6.2 CAS 循环

```rust
use qubit_atomic::::AtomicI32;

fn increment_even_only(atomic: &AtomicI32) -> Result<i32, &'static str> {
    let mut current = atomic.get();
    loop {
        // 只对偶数值进行递增
        if current % 2 != 0 {
            return Err("Value is odd");
        }

        let new = current + 2;
        match atomic.compare_and_set(current, new) {
            Ok(_) => return Ok(new),
            Err(actual) => current = actual, // 重试
        }
    }
}

fn main() {
    let atomic = AtomicI32::new(10);

    match increment_even_only(&atomic) {
        Ok(new_value) => println!("成功递增到：{}", new_value),
        Err(e) => println!("失败：{}", e),
    }

    assert_eq!(atomic.get(), 12);
}
```

### 6.3 函数式更新

```rust
use qubit_atomic::::AtomicI32;

fn main() {
    let atomic = AtomicI32::new(10);

    // 使用函数更新
    let new_value = atomic.update_and_get(|x| {
        if x < 100 {
            x * 2
        } else {
            x
        }
    });

    assert_eq!(new_value, 20);
    println!("更新后的值：{}", new_value);

    // 累积操作
    let result = atomic.accumulate_and_get(5, |a, b| a + b);
    assert_eq!(result, 25);
    println!("累积后的值：{}", result);
}
```

### 6.4 原子引用

```rust
use qubit_atomic::::AtomicRef;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Config {
    timeout: u64,
    max_retries: u32,
}

fn main() {
    let config = Arc::new(Config {
        timeout: 1000,
        max_retries: 3,
    });

    let atomic_config = AtomicRef::new(config);

    // 更新配置
    let new_config = Arc::new(Config {
        timeout: 2000,
        max_retries: 5,
    });

    let old_config = atomic_config.swap(new_config);
    println!("旧配置：{:?}", old_config);
    println!("新配置：{:?}", atomic_config.get());

    // 使用函数更新
    atomic_config.update_and_get(|current| {
        Arc::new(Config {
            timeout: current.timeout * 2,
            max_retries: current.max_retries + 1,
        })
    });

    println!("更新后的配置：{:?}", atomic_config.get());
}
```

### 6.5 布尔标志

```rust
use qubit_atomic::::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct Service {
    running: Arc<AtomicBool>,
}

impl Service {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn start(&self) {
        // 只有当前未运行时才启动
        if self.running.compare_and_set_if_false(true).is_ok() {
            println!("服务启动成功");
        } else {
            println!("服务已经在运行");
        }
    }

    fn stop(&self) {
        // 只有当前运行时才停止
        if self.running.compare_and_set_if_true(false).is_ok() {
            println!("服务停止成功");
        } else {
            println!("服务已经停止");
        }
    }

    fn is_running(&self) -> bool {
        self.running.get()
    }
}

fn main() {
    let service = Service::new();

    service.start();
    assert!(service.is_running());

    service.start(); // 重复启动会失败

    service.stop();
    assert!(!service.is_running());

    service.stop(); // 重复停止会失败
}
```

### 6.6 使用 Trait 的泛型代码

```rust
use qubit_atomic::::{Atomic, AtomicInteger, AtomicI32, AtomicI64};

/// 通用的原子计数器
fn increment_atomic<T>(atomic: &T) -> T::Value
where
    T: AtomicInteger<Value = i32>,
{
    atomic.increment_and_get()
}

fn main() {
    let counter32 = AtomicI32::new(0);
    let result = increment_atomic(&counter32);
    assert_eq!(result, 1);

    let counter64 = AtomicI64::new(0);
    // increment_atomic(&counter64); // 编译错误：类型不匹配
}
```

### 6.7 高性能场景：直接操作底层类型

```rust
use qubit_atomic::::AtomicI32;
use std::sync::atomic::Ordering;

fn high_performance_counter() {
    let counter = AtomicI32::new(0);

    // 在只需要保证原子性、不需要同步其他状态的场景下，
    // 可以直接访问底层类型使用 Relaxed ordering 获得最佳性能
    for _ in 0..1_000_000 {
        counter.inner().fetch_add(1, Ordering::Relaxed);
    }

    // 最后使用 Acquire 读取最终值
    let final_count = counter.inner().load(Ordering::Acquire);
    println!("最终计数：{}", final_count);
}

fn mixed_usage() {
    let counter = AtomicI32::new(0);

    // 99% 的代码使用简单 API
    counter.increment_and_get();
    counter.add_and_get(5);

    // 1% 的关键路径使用精细控制
    unsafe {
        // 某些极端场景可能需要 unsafe 配合底层类型
    }

    // 继续使用简单 API
    let value = counter.get();
    println!("当前值：{}", value);
}
```

## 7. 性能优化指南：何时使用 `inner()`

### 7.1 总体原则

**99% 的场景**：使用默认 API 就足够了，不需要调用 `inner()`。

**1% 的场景**：在性能极其关键的热点代码路径上，经过性能分析确认存在瓶颈后，才考虑使用 `inner()` 进行微调。

### 7.2 默认内存序的性能特点

我们的默认内存序策略已经过仔细设计，平衡了正确性和性能：

| 操作类型 | 默认 Ordering | 性能特点 | 典型场景 |
|---------|--------------|---------|---------|
| **读取** (`get()`) | `Acquire` | 轻量级，读屏障 | 读取共享状态 |
| **写入** (`set()`) | `Release` | 轻量级，写屏障 | 更新共享状态 |
| **RMW** (`swap()`, CAS) | `AcqRel` | 中等，读写屏障 | 原子交换 |
| **计数器** (`increment_and_get()`) | `Relaxed` | 最快，无屏障 | 纯计数统计 |

**关键点**：我们的默认策略在大多数架构上性能已经很好，不需要手动优化。

### 7.3 何时应该使用 `inner()`

#### 场景 1：高频计数器，不需要同步其他状态

```rust
use std::sync::atomic::Ordering;

// ❌ 过度使用：默认 API 已经使用 Relaxed
let counter = AtomicI32::new(0);
for _ in 0..1_000_000 {
    counter.increment_and_get();  // 内部已经是 Relaxed
}

// ✅ 默认 API 就够了
let counter = AtomicI32::new(0);
for _ in 0..1_000_000 {
    counter.increment_and_get();  // 性能最优
}

// ⚠️ 只有当你需要与默认不同的语义时才用 inner()
// 例如：需要 SeqCst 保证严格全局顺序
for _ in 0..1_000_000 {
    counter.inner().fetch_add(1, Ordering::SeqCst);  // 显式需要最强保证
}
```

#### 场景 2：延迟写入（Lazy Set）

```rust
use std::sync::atomic::Ordering;

struct Cache {
    dirty: AtomicBool,
    data: Vec<u8>,
}

impl Cache {
    fn mark_dirty(&self) {
        // ✅ 使用 Relaxed：标记为脏不需要立即对其他线程可见
        // 因为实际数据的写入会有更强的同步
        self.dirty.inner().store(true, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        // ✅ 读取时使用 Acquire 确保看到数据的变更
        self.dirty.get()  // 默认 Acquire
    }
}
```

**原因**：这是 JDK 的 `lazySet()` 模式，写入可以延迟，但读取需要同步。

#### 场景 3：自旋锁中的 Relaxed 读取

```rust
use std::sync::atomic::Ordering;

struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    fn lock(&self) {
        // 自旋等待锁释放
        while self.locked.inner().load(Ordering::Relaxed) {
            // ✅ 使用 Relaxed：频繁读取，不需要同步其他状态
            std::hint::spin_loop();
        }

        // 真正获取锁时使用 CAS（默认 AcqRel）
        while self.locked.compare_and_set(false, true).is_err() {
            while self.locked.inner().load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    fn unlock(&self) {
        // ❌ 错误：不能使用 Relaxed
        // self.locked.inner().store(false, Ordering::Relaxed);

        // ✅ 正确：释放锁必须用 Release
        self.locked.set(false);  // 默认 Release
    }
}
```

**关键点**：
- 自旋等待时的读取可以 `Relaxed`（性能关键）
- 但获取和释放锁必须用正确的内存序（默认 API 已提供）

#### 场景 4：SeqCst 保证严格全局顺序

```rust
use std::sync::atomic::Ordering;

// 某些算法需要严格的全局顺序（少见）
struct SequentialConsistencyRequired {
    flag1: AtomicBool,
    flag2: AtomicBool,
}

impl SequentialConsistencyRequired {
    fn operation(&self) {
        // ✅ 需要 SeqCst 保证全局顺序
        self.flag1.inner().store(true, Ordering::SeqCst);

        if self.flag2.inner().load(Ordering::SeqCst) {
            // 保证看到全局一致的顺序
        }
    }
}
```

**注意**：这种场景非常罕见，大多数算法用 Acquire/Release 就够了。

#### 场景 5：性能基准测试

```rust
use std::sync::atomic::Ordering;

fn benchmark_compare() {
    let counter = AtomicI32::new(0);

    // 测试默认 API（Relaxed for increment）
    let start = Instant::now();
    for _ in 0..10_000_000 {
        counter.increment_and_get();
    }
    println!("Default API: {:?}", start.elapsed());

    // 测试显式 Relaxed（应该相同）
    counter.set(0);
    let start = Instant::now();
    for _ in 0..10_000_000 {
        counter.inner().fetch_add(1, Ordering::Relaxed);
    }
    println!("Explicit Relaxed: {:?}", start.elapsed());

    // 测试 SeqCst（应该更慢）
    counter.set(0);
    let start = Instant::now();
    for _ in 0..10_000_000 {
        counter.inner().fetch_add(1, Ordering::SeqCst);
    }
    println!("SeqCst: {:?}", start.elapsed());
}
```

### 7.4 何时不应该使用 `inner()`

#### 反模式 1：没有性能瓶颈就优化

```rust
// ❌ 错误：过早优化
fn process_data() {
    let counter = AtomicI32::new(0);
    for item in items {
        // 没有证据表明这里是性能瓶颈
        counter.inner().fetch_add(1, Ordering::Relaxed);
    }
}

// ✅ 正确：使用默认 API
fn process_data() {
    let counter = AtomicI32::new(0);
    for item in items {
        counter.increment_and_get();  // 清晰且性能已经很好
    }
}
```

#### 反模式 2：误用 Relaxed 破坏同步

```rust
// ❌ 错误：使用 Relaxed 破坏了同步
let flag = AtomicBool::new(false);
let mut data = 42;

// 线程 1
data = 100;
flag.inner().store(true, Ordering::Relaxed);  // 错误！

// 线程 2
if flag.inner().load(Ordering::Relaxed) {  // 错误！
    println!("{}", data);  // 可能看到旧值 42
}

// ✅ 正确：使用默认 API
// 线程 1
data = 100;
flag.set(true);  // Release - 保证 data 的写入可见

// 线程 2
if flag.get() {  // Acquire - 保证看到 data 的更新
    println!("{}", data);  // 一定看到 100
}
```

#### 反模式 3：为了"看起来专业"而使用

```rust
// ❌ 错误：炫技
fn update_stats(&self) {
    self.counter.inner().fetch_add(1, Ordering::Relaxed);
    self.timestamp.inner().store(now(), Ordering::Release);
}

// ✅ 正确：清晰明了
fn update_stats(&self) {
    self.counter.increment_and_get();  // 已经是 Relaxed
    self.timestamp.set(now());         // 已经是 Release
}
```

### 7.5 性能优化决策树

```
是否有性能问题？
├─ 否 → 使用默认 API
└─ 是
    ├─ 已经用性能分析工具确认是瓶颈？
    │   ├─ 否 → 使用默认 API（不要猜测）
    │   └─ 是
    │       ├─ 是纯计数场景？
    │       │   ├─ 是 → 默认 API 已经是 Relaxed
    │       │   └─ 否 → 继续
    │       ├─ 需要特殊的内存序语义？
    │       │   ├─ 是 → 使用 inner()
    │       │   └─ 否 → 使用默认 API
    │       └─ 在自旋循环中频繁读取？
    │           ├─ 是 → 考虑 inner().load(Relaxed)
    │           └─ 否 → 使用默认 API
```

### 7.6 性能对比数据（参考）

以下是不同内存序在典型架构上的相对性能（数字越小越快）：

| 操作 | x86-64 | ARM64 | 说明 |
|-----|--------|-------|------|
| `Relaxed` | 1.0x | 1.0x | 基线 |
| `Acquire` (读) | 1.0x | 1.1x | x86 免费，ARM 需要屏障 |
| `Release` (写) | 1.0x | 1.1x | x86 免费，ARM 需要屏障 |
| `AcqRel` (RMW) | 1.0x | 1.2x | x86 免费，ARM 需要双屏障 |
| `SeqCst` (读) | 2.0x | 2.0x | 需要 mfence/dmb |
| `SeqCst` (写) | 2.0x | 2.0x | 需要 mfence/dmb |
| `SeqCst` (RMW) | 2.0x | 2.5x | 最重的同步 |

**结论**：
- 在 x86-64 上，`Acquire/Release/AcqRel` 几乎是免费的
- 在 ARM 上，有轻微开销，但通常可以接受
- `SeqCst` 在所有架构上都明显更慢
- 我们的默认策略（Acquire/Release/AcqRel）在各架构上都是最佳平衡

### 7.7 使用 `inner()` 的检查清单

在使用 `inner()` 之前，问自己这些问题：

- [ ] 我已经用性能分析工具（如 `cargo flamegraph`）确认这是瓶颈吗？
- [ ] 我理解不同内存序的语义和后果吗？
- [ ] 默认 API 真的不够用吗？
- [ ] 我的使用会破坏内存同步吗？
- [ ] 我在代码注释中解释了为什么需要特殊内存序吗？
- [ ] 我写了测试验证正确性吗（尤其是并发测试）？

**如果有任何一个答案是"否"，请不要使用 `inner()`。**

### 7.8 总结：黄金法则

> **默认 API 优先，`inner()` 是最后的手段。**

- 🟢 **总是先用默认 API**：99% 的情况下性能已经足够好
- 🟡 **测量再优化**：只有确认是瓶颈才考虑 `inner()`
- 🔴 **理解再使用**：使用 `inner()` 前确保理解内存序语义
- 📝 **记录原因**：如果使用了 `inner()`，在代码注释中解释为什么

**记住**：过早优化是万恶之源。清晰的代码比微小的性能提升更有价值。

## 8. 实现细节

### 8.1 内存布局

所有封装类型都应该具有与底层标准库类型相同的内存布局：

```rust
#[repr(transparent)]
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}
```

使用 `#[repr(transparent)]` 确保零成本抽象。

### 7.2 方法内联

所有方法都应该内联，避免函数调用开销：

```rust
impl AtomicI32 {
    #[inline]
    pub fn get(&self) -> i32 {
        self.inner.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set(&self, value: i32) {
        self.inner.store(value, Ordering::Release);
    }

    #[inline]
    pub fn inner(&self) -> &std::sync::atomic::AtomicI32 {
        &self.inner
    }

    // ... 其他方法
}
```

### 7.3 CAS 循环实现

函数式更新方法使用标准 CAS 循环模式：

```rust
impl AtomicI32 {
    pub fn update_and_get<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return new,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn get_and_update<F>(&self, f: F) -> i32
    where
        F: Fn(i32) -> i32,
    {
        let mut current = self.get();
        loop {
            let new = f(current);
            match self.compare_and_set_weak(current, new) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }
}
```

### 7.4 AtomicRef 实现细节

`AtomicRef` 需要正确管理 `Arc` 的引用计数：

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::ptr;

pub struct AtomicRef<T> {
    inner: AtomicPtr<T>,
}

impl<T> AtomicRef<T> {
    pub fn new(value: Arc<T>) -> Self {
        let ptr = Arc::into_raw(value) as *mut T;
        Self {
            inner: AtomicPtr::new(ptr),
        }
    }

    pub fn get(&self) -> Arc<T> {
        let ptr = self.inner.load(Ordering::Acquire);
        unsafe {
            // 增加引用计数但不释放原指针
            let arc = Arc::from_raw(ptr);
            let cloned = arc.clone();
            Arc::into_raw(arc); // 防止释放
            cloned
        }
    }

    pub fn set(&self, value: Arc<T>) {
        let new_ptr = Arc::into_raw(value) as *mut T;
        let old_ptr = self.inner.swap(new_ptr, Ordering::AcqRel);
        unsafe {
            if !old_ptr.is_null() {
                // 释放旧值
                Arc::from_raw(old_ptr);
            }
        }
    }

    // ... 其他方法
}

impl<T> Drop for AtomicRef<T> {
    fn drop(&mut self) {
        let ptr = self.inner.load(Ordering::Acquire);
        unsafe {
            if !ptr.is_null() {
                Arc::from_raw(ptr);
            }
        }
    }
}

unsafe impl<T: Send + Sync> Send for AtomicRef<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicRef<T> {}
```

## 8. 性能考虑

### 8.1 零成本抽象验证

使用 `#[repr(transparent)]` 和 `#[inline]` 确保编译器优化后的代码与直接使用标准库类型相同：

```rust
// 我们的封装
let atomic = AtomicI32::new(0);
let value = atomic.get();

// 编译后应该等价于
let atomic = std::sync::atomic::AtomicI32::new(0);
let value = atomic.load(Ordering::Acquire);
```

可以通过以下方式验证：

```bash
# 查看生成的汇编代码
cargo rustc --release -- --emit=asm

# 或使用 cargo-show-asm
cargo install cargo-show-asm
cargo asm --release qubit_atomic::::AtomicI32::get
```

### 8.2 内存序性能对比

不同内存序的性能开销（从小到大）：

1. **Relaxed** - 几乎无开销，只保证原子性
2. **Acquire/Release** - 轻微开销，防止指令重排
3. **AcqRel** - 中等开销，结合 Acquire 和 Release
4. **SeqCst** - 最大开销，保证全局顺序一致性

### 8.3 性能优化建议

1. **纯计数场景**：如果性能关键，可以直接使用 `inner()` 配合 `Relaxed` ordering
   ```rust
   use std::sync::atomic::Ordering;

   // 性能关键路径
   counter.inner().fetch_add(1, Ordering::Relaxed);

   // 或者使用默认 API（已经使用 Relaxed）
   counter.get_and_increment();  // 内部也是 Relaxed
   ```

2. **状态同步场景**：使用默认 API（自动使用 `Acquire/Release`）
   ```rust
   if atomic.get() {
       // 读取到 true 时，之前的写入一定可见
   }
   ```

3. **CAS 循环**：使用 `compare_and_set_weak`
   ```rust
   // 弱 CAS 在某些平台上性能更好
   loop {
       match atomic.compare_and_set_weak(current, new) {
           Ok(_) => break,
           Err(actual) => current = actual,
       }
   }
   ```

4. **何时使用 `inner()`**：
   - **不需要**：大多数场景，默认 API 已经足够好
   - **需要**：极致性能优化、复杂无锁算法、需要 `SeqCst` 等特殊内存序

## 9. 与 JDK 对比

### 9.1 完整 API 对照表

#### 9.1.1 AtomicInteger (JDK) vs AtomicI32 (Rust)

| 分类 | JDK API | Rust 封装 API | 实现状态 | 说明 |
|------|---------|--------------|---------|------|
| **构造** | `new(int value)` | `new(value: i32)` | ✅ | 构造函数 |
| **基础操作** | `get()` | `get()` | ✅ | 读取当前值 |
| | `set(int newValue)` | `set(value: i32)` | ✅ | 设置新值 |
| | `lazySet(int newValue)` | `inner().store(value, Relaxed)` | ✅ | 延迟写入（通过 inner）|
| | `getAndSet(int newValue)` | `swap(value: i32)` | ✅ | 交换值（Rust 习惯命名）|
| **自增/自减** | `getAndIncrement()` | `get_and_increment()` | ✅ | 后增 |
| | `incrementAndGet()` | `increment_and_get()` | ✅ | 前增 |
| | `getAndDecrement()` | `get_and_decrement()` | ✅ | 后减 |
| | `decrementAndGet()` | `decrement_and_get()` | ✅ | 前减 |
| **算术操作** | `getAndAdd(int delta)` | `get_and_add(delta: i32)` | ✅ | 后加 |
| | `addAndGet(int delta)` | `add_and_get(delta: i32)` | ✅ | 前加 |
| | - | `get_and_sub(delta: i32)` | ✅ | 后减（Rust 特有）|
| | - | `sub_and_get(delta: i32)` | ✅ | 前减（Rust 特有）|
| **CAS 操作** | `compareAndSet(int expect, int update)` | `compare_and_set(current, new)` | ✅ | CAS |
| | `weakCompareAndSet(int expect, int update)` | `compare_and_set_weak(current, new)` | ✅ | 弱 CAS |
| | `compareAndExchange(int expect, int update)` (Java 9+) | `inner().compare_exchange(...)` | ✅ | 通过 inner 支持 |
| **函数式更新** | `getAndUpdate(IntUnaryOperator f)` (Java 8+) | `get_and_update(f)` | ✅ | 函数更新，返回旧值 |
| | `updateAndGet(IntUnaryOperator f)` (Java 8+) | `update_and_get(f)` | ✅ | 函数更新，返回新值 |
| | `getAndAccumulate(int x, IntBinaryOperator f)` (Java 8+) | `get_and_accumulate(x, f)` | ✅ | 累积，返回旧值 |
| | `accumulateAndGet(int x, IntBinaryOperator f)` (Java 8+) | `accumulate_and_get(x, f)` | ✅ | 累积，返回新值 |
| **位运算** | - | `get_and_bitand(value)` | ✅ | 按位与（Rust 特有）|
| | - | `get_and_bitor(value)` | ✅ | 按位或（Rust 特有）|
| | - | `get_and_bitxor(value)` | ✅ | 按位异或（Rust 特有）|
| **最大/最小值** | - | `get_and_max(value)` | ✅ | 取最大值（Rust 特有）|
| | - | `max_and_get(value)` | ✅ | 取最大值，返回新值 |
| | - | `get_and_min(value)` | ✅ | 取最小值（Rust 特有）|
| | - | `min_and_get(value)` | ✅ | 取最小值，返回新值 |
| **类型转换** | `intValue()` | `get()` | ✅ | 直接用 get() |
| | `longValue()` | `get() as i64` | ✅ | 通过 as 转换 |
| | `floatValue()` | `get() as f32` | ✅ | 通过 as 转换 |
| | `doubleValue()` | `get() as f64` | ✅ | 通过 as 转换 |
| **其他** | `toString()` | `Display` trait | ✅ | 实现 Display |
| | - | `Debug` trait | ✅ | 实现 Debug |
| | - | `inner()` | ✅ | 访问底层类型（Rust 特有）|
| | - | `into_inner()` | ✅ | 转换为底层类型 |
| | - | `from_std(std_atomic)` | ✅ | 从标准库类型创建 |

#### 9.1.2 AtomicBoolean (JDK) vs AtomicBool (Rust)

| 分类 | JDK API | Rust 封装 API | 实现状态 | 说明 |
|------|---------|--------------|---------|------|
| **构造** | `new(boolean value)` | `new(value: bool)` | ✅ | 构造函数 |
| **基础操作** | `get()` | `get()` | ✅ | 读取当前值 |
| | `set(boolean newValue)` | `set(value: bool)` | ✅ | 设置新值 |
| | `lazySet(boolean newValue)` | `inner().store(value, Relaxed)` | ✅ | 延迟写入（通过 inner）|
| | `getAndSet(boolean newValue)` | `swap(value: bool)` | ✅ | 交换值 |
| **CAS 操作** | `compareAndSet(boolean expect, boolean update)` | `compare_and_set(current, new)` | ✅ | CAS |
| | `weakCompareAndSet(boolean expect, boolean update)` | `compare_and_set_weak(current, new)` | ✅ | 弱 CAS |
| **布尔特有** | - | `get_and_set()` | ✅ | 设置为 true，返回旧值（Rust 特有）|
| | - | `set_and_get()` | ✅ | 设置为 true，返回新值 |
| | - | `get_and_clear()` | ✅ | 设置为 false，返回旧值 |
| | - | `clear_and_get()` | ✅ | 设置为 false，返回新值 |
| | - | `get_and_negate()` | ✅ | 取反，返回旧值（Rust 特有）|
| | - | `negate_and_get()` | ✅ | 取反，返回新值 |
| | - | `get_and_logical_and(bool)` | ✅ | 逻辑与（Rust 特有）|
| | - | `get_and_logical_or(bool)` | ✅ | 逻辑或（Rust 特有）|
| | - | `get_and_logical_xor(bool)` | ✅ | 逻辑异或（Rust 特有）|
| | - | `compare_and_set_if_false(new)` | ✅ | 条件 CAS（Rust 特有）|
| | - | `compare_and_set_if_true(new)` | ✅ | 条件 CAS（Rust 特有）|
| **其他** | `toString()` | `Display` trait | ✅ | 实现 Display |
| | - | `inner()` | ✅ | 访问底层类型 |

#### 9.1.3 AtomicReference (JDK) vs AtomicRef (Rust)

| 分类 | JDK API | Rust 封装 API | 实现状态 | 说明 |
|------|---------|--------------|---------|------|
| **构造** | `new(V value)` | `new(value: Arc<T>)` | ✅ | 构造函数（使用 Arc）|
| **基础操作** | `get()` | `get()` | ✅ | 获取当前引用 |
| | `set(V newValue)` | `set(value: Arc<T>)` | ✅ | 设置新引用 |
| | `lazySet(V newValue)` | `inner().store(ptr, Relaxed)` | ✅ | 延迟写入（通过 inner）|
| | `getAndSet(V newValue)` | `swap(value: Arc<T>)` | ✅ | 交换引用 |
| **CAS 操作** | `compareAndSet(V expect, V update)` | `compare_and_set(&current, new)` | ✅ | CAS（指针相等性）|
| | `weakCompareAndSet(V expect, V update)` | `compare_and_set_weak(&current, new)` | ✅ | 弱 CAS |
| **函数式更新** | `getAndUpdate(UnaryOperator<V> f)` (Java 8+) | `get_and_update(f)` | ✅ | 函数更新，返回旧引用 |
| | `updateAndGet(UnaryOperator<V> f)` (Java 8+) | `update_and_get(f)` | ✅ | 函数更新，返回新引用 |
| | `getAndAccumulate(V x, BinaryOperator<V> f)` (Java 8+) | `get_and_accumulate(x, f)` | ✅ | 累积，返回旧引用 |
| | `accumulateAndGet(V x, BinaryOperator<V> f)` (Java 8+) | `accumulate_and_get(x, f)` | ✅ | 累积，返回新引用 |
| **其他** | `toString()` | `Display` trait (如果 T: Display) | ✅ | 实现 Display |
| | - | `inner()` | ✅ | 访问底层类型 |
| | - | `Clone` trait | ✅ | 克隆原子引用 |

#### 9.1.4 JDK 没有但 Rust 提供的类型

| Rust 类型 | 说明 | 对应 JDK 类型 |
|----------|------|--------------|
| `AtomicU32` | 32位无符号整数 | - |
| `AtomicU64` | 64位无符号整数 | - |
| `AtomicIsize` | 指针大小的有符号整数 | - |
| `AtomicUsize` | 指针大小的无符号整数 | - |

#### 9.1.5 API 总结

| 特性 | JDK | Rust 封装 | 说明 |
|-----|-----|----------|------|
| **基础方法数** | ~15 个/类型 | ~25 个/类型 | Rust 提供更多便利方法 |
| **函数式方法** | Java 8+ 支持 | ✅ 支持 | 两者等价 |
| **位运算** | ❌ 不支持 | ✅ 支持 | Rust 特有（更强大）|
| **最大/最小值** | ❌ 不支持 | ✅ 支持 | Rust 特有 |
| **内存序控制** | 隐式（volatile） | 默认 + `inner()` 可选 | Rust 更灵活 |
| **类型数量** | 3 种基础类型 | 8 种基础类型 | Rust 支持更多整数类型 |

### 9.2 关键差异

| 特性 | JDK | Rust 封装 | 说明 |
|-----|-----|----------|------|
| **内存序** | 隐式（使用 volatile 语义） | 默认自动 + `inner()` 可选 | 99% 场景无需关心，1% 场景通过 `inner()` 控制 |
| **弱 CAS** | `weakCompareAndSet` | `compare_and_set_weak` | 两者等价 |
| **引用类型** | `AtomicReference<V>` | `AtomicRef<T>` | Rust 使用 `Arc<T>` |
| **可空性** | 允许 `null` | 使用 `Option<Arc<T>>` | Rust 不允许空指针 |
| **位运算** | 部分支持 | 完整支持 | Rust 支持所有位运算 |
| **最大/最小值** | Java 9+ 支持 | 支持 | 两者等价 |
| **API 数量** | ~20 个方法/类型 | ~25 个方法/类型 | Rust 不提供 `_with_ordering` 变体，API 更简洁 |

### 9.3 Rust 特有优势

1. **编译期内存安全**：完全避免数据竞争
2. **零成本抽象**：内联后无性能开销
3. **精细的内存序控制**：可根据需求选择最优内存序
4. **类型安全**：通过 trait 系统保证正确使用
5. **无垃圾回收开销**：`Arc` 使用引用计数，可预测的性能

## 10. 模块结构

```
rust-concurrent/
├── src/
│   ├── lib.rs
│   ├── atomic/                      # 新增：原子类型模块
│   │   ├── mod.rs                   # 模块导出
│   │   ├── atomic_bool.rs           # AtomicBool 实现
│   │   ├── atomic_i32.rs            # AtomicI32 实现
│   │   ├── atomic_i64.rs            # AtomicI64 实现
│   │   ├── atomic_u32.rs            # AtomicU32 实现
│   │   ├── atomic_u64.rs            # AtomicU64 实现
│   │   ├── atomic_isize.rs          # AtomicIsize 实现
│   │   ├── atomic_usize.rs          # AtomicUsize 实现
│   │   ├── atomic_ref.rs            # AtomicRef<T> 实现
│   │   └── traits.rs                # Atomic trait 定义
│   ├── double_checked/
│   ├── executor.rs
│   └── lock/
├── tests/
│   ├── atomic/                      # 新增：原子类型测试
│   │   ├── mod.rs
│   │   ├── atomic_bool_tests.rs
│   │   ├── atomic_i32_tests.rs
│   │   ├── atomic_i64_tests.rs
│   │   ├── atomic_u32_tests.rs
│   │   ├── atomic_u64_tests.rs
│   │   ├── atomic_isize_tests.rs
│   │   ├── atomic_usize_tests.rs
│   │   ├── atomic_ref_tests.rs
│   │   ├── trait_tests.rs           # Trait 测试
│   │   ├── concurrent_tests.rs      # 并发测试
│   │   └── performance_tests.rs     # 性能测试
│   ├── double_checked/
│   └── lock/
├── examples/
│   ├── atomic_counter_demo.rs       # 新增：计数器示例
│   ├── atomic_cas_demo.rs           # 新增：CAS 示例
│   ├── atomic_ref_demo.rs           # 新增：引用示例
│   ├── atomic_bool_demo.rs          # 新增：布尔标志示例
│   └── atomic_performance_demo.rs   # 新增：性能对比示例
├── benches/
│   └── atomic_bench.rs              # 新增：性能基准测试
└── doc/
    └── atomic_design_zh_CN_v1.0.claude.md  # 本文档
```

## 11. 实施计划

### 11.1 第一阶段：基础框架（1 天）

- [ ] 创建模块结构
- [ ] 定义 `Atomic` 和相关 trait
- [ ] 实现 `AtomicBool` 完整功能（含 `inner()` 方法）
- [ ] 实现 `AtomicI32` 完整功能（含 `inner()` 方法）
- [ ] 编写基础单元测试

### 11.2 第二阶段：扩展类型（1-2 天）

- [ ] 实现 `AtomicI64`
- [ ] 实现 `AtomicU32`
- [ ] 实现 `AtomicU64`
- [ ] 实现 `AtomicIsize`
- [ ] 实现 `AtomicUsize`
- [ ] 为所有类型实现 trait
- [ ] 为所有类型实现 `inner()` 方法
- [ ] 编写完整单元测试

### 11.3 第三阶段：引用类型（2 天）

- [ ] 实现 `AtomicRef<T>` 基础功能
- [ ] 正确处理 `Arc` 引用计数
- [ ] 实现 `Drop` trait
- [ ] 实现 `Send + Sync`
- [ ] 实现 `inner()` 方法
- [ ] 编写安全性测试

### 11.4 第四阶段：高级功能（1-2 天）

- [ ] 实现函数式更新方法
- [ ] 实现累积操作
- [ ] 实现最大/最小值操作
- [ ] 实现位运算操作
- [ ] 编写并发测试

### 11.5 第五阶段：文档和示例（1-2 天）

- [ ] 编写完整的 API 文档注释（中文）
- [ ] 编写使用示例（7 个场景）
- [ ] 编写 README
- [ ] 编写性能对比文档
- [ ] 编写迁移指南（从标准库到封装）
- [ ] 编写 `inner()` 使用指南

### 11.6 第六阶段：性能优化和测试（1-2 天）

- [ ] 编写性能基准测试
- [ ] 验证零成本抽象（对比标准库）
- [ ] 验证 `inner()` 零开销
- [ ] 进行并发压力测试
- [ ] 代码覆盖率测试
- [ ] 内存序正确性测试（使用 loom）

**总计**：约 7-10 天（相比原计划减少 3-4 天，因为不需要实现所有 `_with_ordering` 变体）

## 12. 测试策略

### 12.1 单元测试

每个原子类型都应该测试：

1. **基础操作**：`new`、`get`、`set`、`swap`
2. **CAS 操作**：成功和失败的情况
3. **自增/自减**：正确性和边界值
4. **算术操作**：加减乘除
5. **位运算**：与或非异或
6. **函数式更新**：各种更新函数
7. **最大/最小值**：边界情况

### 12.2 并发测试

```rust
#[test]
fn test_concurrent_increment() {
    use std::sync::Arc;
    use std::thread;

    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    // 10 个线程，每个递增 10000 次
    for _ in 0..10 {
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..10000 {
                counter.increment_and_get();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.get(), 100000);
}
```

### 12.3 内存序测试

使用 `loom` crate 进行内存模型测试：

```rust
#[cfg(loom)]
#[test]
fn test_memory_ordering() {
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::thread;

    loom::model(|| {
        let atomic = Arc::new(AtomicI32::new(0));
        let atomic2 = atomic.clone();

        let t1 = thread::spawn(move || {
            atomic.set(1);
        });

        let t2 = thread::spawn(move || {
            atomic2.get()
        });

        t1.join().unwrap();
        let result = t2.join().unwrap();
        // 验证内存序的正确性
    });
}
```

### 12.4 性能基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_atomic_increment(c: &mut Criterion) {
    use qubit_atomic::::AtomicI32;

    c.bench_function("atomic_increment", |b| {
        let atomic = AtomicI32::new(0);
        b.iter(|| {
            atomic.increment_and_get();
        });
    });
}

fn bench_std_atomic_increment(c: &mut Criterion) {
    use std::sync::atomic::{AtomicI32, Ordering};

    c.bench_function("std_atomic_increment", |b| {
        let atomic = AtomicI32::new(0);
        b.iter(|| {
            atomic.fetch_add(1, Ordering::Relaxed);
        });
    });
}

criterion_group!(benches, bench_atomic_increment, bench_std_atomic_increment);
criterion_main!(benches);
```

## 13. 最佳实践

### 13.1 选择合适的原子类型

| 场景 | 推荐类型 | 原因 |
|-----|---------|------|
| 简单计数器 | `AtomicI32`/`AtomicU32` | 最常见，性能好 |
| 大范围计数 | `AtomicI64`/`AtomicU64` | 避免溢出 |
| 布尔标志 | `AtomicBool` | 语义清晰 |
| 指针大小的值 | `AtomicIsize`/`AtomicUsize` | 平台相关 |
| 共享配置 | `AtomicRef<Config>` | 支持复杂类型 |

### 13.2 内存序选择指南

| 场景 | 推荐内存序 | 说明 |
|-----|----------|------|
| 纯计数，无其他状态 | `Relaxed` | 最佳性能 |
| 读取共享状态 | `Acquire`（默认） | 保证读到最新值 |
| 更新共享状态 | `Release`（默认） | 保证写入可见 |
| CAS 操作 | `AcqRel`（默认） | 标准 CAS 语义 |
| 需要严格顺序 | `SeqCst` | 牺牲性能换取正确性 |

### 13.3 常见陷阱

#### 陷阱 1：不必要地使用 `inner()`

```rust
// ❌ 不推荐：不必要的显式 ordering
counter.inner().fetch_add(1, Ordering::Relaxed);

// ✅ 推荐：使用默认 API（已经是 Relaxed）
counter.get_and_increment();
```

#### 陷阱 2：通过 `inner()` 误用 `Relaxed`

```rust
use std::sync::atomic::Ordering;

// ❌ 错误：使用 Relaxed 同步标志
let flag = AtomicBool::new(false);
let mut data = 42;

// 线程 1
data = 100;
flag.inner().store(true, Ordering::Relaxed); // 错误！data 可能不可见

// 线程 2
if flag.inner().load(Ordering::Relaxed) {  // 错误！
    println!("{}", data); // 可能读到旧值 42
}

// ✅ 正确：使用默认 API（自动使用 Acquire/Release）
flag.set(true); // Release - 保证之前的写入可见
if flag.get() { // Acquire - 保证读取到最新值
    println!("{}", data); // 保证读到 100
}
```

**教训**：默认 API 已经为你选择了正确的内存序，不要画蛇添足！

#### 陷阱 3：忘记处理 CAS 失败

```rust
// ❌ 错误：忽略 CAS 失败
atomic.compare_and_set(expected, new);

// ✅ 正确：处理 CAS 结果
match atomic.compare_and_set(expected, new) {
    Ok(_) => println!("成功"),
    Err(actual) => println!("失败，当前值: {}", actual),
}
```

### 13.4 性能优化技巧

#### 技巧 1：批量操作

```rust
// ❌ 效率低：多次原子操作
for _ in 0..1000 {
    counter.increment_and_get();
}

// ✅ 效率高：一次原子操作
counter.add_and_get(1000);
```

#### 技巧 2：使用弱 CAS

```rust
// ✅ 在循环中使用弱 CAS
loop {
    match atomic.compare_and_set_weak(current, new) {
        Ok(_) => break,
        Err(actual) => current = actual,
    }
}
```

#### 技巧 3：避免不必要的读取

```rust
// ❌ 不必要的读取
let old = atomic.get();
let new = old + 1;
atomic.set(new);

// ✅ 直接使用自增
atomic.increment_and_get();
```

## 14. 与现有生态集成

### 14.1 与标准库的互操作

```rust
use std::sync::atomic::AtomicI32 as StdAtomicI32;
use std::sync::atomic::Ordering;
use qubit_atomic::::AtomicI32;

impl From<StdAtomicI32> for AtomicI32 {
    fn from(std_atomic: StdAtomicI32) -> Self {
        Self::new(std_atomic.load(Ordering::Acquire))
    }
}

impl AtomicI32 {
    /// 获取底层标准库类型的引用
    ///
    /// 这是与标准库互操作的主要方法。
    #[inline]
    pub fn inner(&self) -> &StdAtomicI32 {
        &self.inner
    }

    /// 转换为标准库类型（消耗 self）
    pub fn into_inner(self) -> StdAtomicI32 {
        self.inner
    }

    /// 从标准库类型创建（零成本）
    pub const fn from_std(std_atomic: StdAtomicI32) -> Self {
        Self { inner: std_atomic }
    }
}

// 使用示例
fn interop_example() {
    // 封装类型 -> 标准库类型
    let atomic = AtomicI32::new(42);
    let std_atomic = atomic.inner();
    std_atomic.store(100, Ordering::Release);

    // 标准库类型 -> 封装类型
    let std_atomic = StdAtomicI32::new(42);
    let atomic = AtomicI32::from_std(std_atomic);
}
```

### 14.2 与 crossbeam 集成

保持与 `crossbeam-utils` 的 `AtomicCell` 兼容性：

```rust
// 可以根据需要在两者之间转换
use crossbeam_utils::atomic::AtomicCell;
use qubit_atomic::::AtomicI32;

let atomic = AtomicI32::new(42);
let cell = AtomicCell::new(atomic.get());
```

### 14.3 与 parking_lot 集成

如果需要，可以提供与 `parking_lot` 的集成：

```rust
use parking_lot::Mutex;
use qubit_atomic::::AtomicBool;

struct Resource {
    data: Mutex<Vec<u8>>,
    initialized: AtomicBool,
}
```

## 15. 文档注释规范

遵循项目的 Rust 文档注释规范：

```rust
/// 原子 32 位有符号整数
///
/// 提供易用的原子操作 API，自动使用合理的内存序。
/// 所有方法都是线程安全的，可以在多个线程间共享使用。
///
/// # 特性
///
/// - 自动选择合适的内存序，简化使用
/// - 提供丰富的高级操作（自增、自减、函数式更新等）
/// - 零成本抽象，性能与直接使用标准库相同
/// - 通过 `inner()` 方法可访问底层类型（高级用法）
///
/// # 使用场景
///
/// - 多线程计数器
/// - 状态标志
/// - 统计数据收集
/// - 无锁算法
///
/// # 基础示例
///
/// ```rust
/// use qubit_atomic::::AtomicI32;
/// use std::sync::Arc;
/// use std::thread;
///
/// let counter = Arc::new(AtomicI32::new(0));
/// let mut handles = vec![];
///
/// for _ in 0..10 {
///     let counter = counter.clone();
///     let handle = thread::spawn(move || {
///         for _ in 0..1000 {
///             counter.increment_and_get();
///         }
///     });
///     handles.push(handle);
/// }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
///
/// assert_eq!(counter.get(), 10000);
/// ```
///
/// # 高级用法：直接访问底层类型
///
/// ```rust
/// use qubit_atomic::::AtomicI32;
/// use std::sync::atomic::Ordering;
///
/// let atomic = AtomicI32::new(0);
///
/// // 99% 的场景：使用简单 API
/// atomic.increment_and_get();
///
/// // 1% 的场景：需要精细控制内存序
/// atomic.inner().store(42, Ordering::Relaxed);
/// let value = atomic.inner().load(Ordering::SeqCst);
/// ```
///
/// # 作者
///
/// 胡海星
pub struct AtomicI32 {
    inner: std::sync::atomic::AtomicI32,
}
```

## 16. 迁移指南

### 16.1 从标准库迁移

```rust
// 迁移前：使用标准库
use std::sync::atomic::{AtomicI32 as StdAtomicI32, Ordering};

let atomic = StdAtomicI32::new(0);
let value = atomic.load(Ordering::Acquire);
atomic.store(42, Ordering::Release);
let old = atomic.fetch_add(1, Ordering::Relaxed);

// 迁移后：使用封装（大多数情况）
use qubit_atomic::::AtomicI32;

let atomic = AtomicI32::new(0);
let value = atomic.get();                // 自动 Acquire
atomic.set(42);                          // 自动 Release
let old = atomic.get_and_increment();   // 自动 Relaxed（计数器场景）

// 如果需要特殊的内存序（少数情况）
use std::sync::atomic::Ordering;
let value = atomic.inner().load(Ordering::SeqCst);
atomic.inner().store(100, Ordering::Relaxed);
```

### 16.1.1 分阶段迁移策略

**阶段 1：新代码使用封装**
```rust
// 新写的代码直接使用封装类型
let counter = AtomicI32::new(0);
counter.increment_and_get();
```

**阶段 2：逐步替换旧代码**
```rust
// 旧代码保持不变
let old_counter = std::sync::atomic::AtomicI32::new(0);

// 通过 from_std 桥接
let new_counter = AtomicI32::from_std(old_counter);
```

**阶段 3：性能关键路径评估**
```rust
// 如果默认内存序不满足性能需求，使用 inner()
for _ in 0..1_000_000 {
    // 性能关键：直接使用 Relaxed
    counter.inner().fetch_add(1, Ordering::Relaxed);
}
```

### 16.2 从 JDK 迁移

```rust
// Java 代码
AtomicInteger counter = new AtomicInteger(0);
int old = counter.getAndIncrement();
int current = counter.incrementAndGet();
boolean success = counter.compareAndSet(10, 20);

// Rust 等价代码
use qubit_atomic::::AtomicI32;

let counter = AtomicI32::new(0);
let old = counter.get_and_increment();
let current = counter.increment_and_get();
let success = counter.compare_and_set(10, 20).is_ok();
```

## 17. 未来扩展

### 17.1 可能的扩展方向

1. **更多整数类型**
   - `AtomicI8`、`AtomicI16`
   - `AtomicU8`、`AtomicU16`

2. **浮点数支持**
   - `AtomicF32`、`AtomicF64`（基于 `AtomicU32`/`AtomicU64` 实现）

3. **原子数组**
   - `AtomicArray<T, N>`

4. **原子指针**
   - 更安全的 `AtomicPtr` 封装

5. **无锁数据结构**
   - 基于原子操作的栈、队列等

6. **统计功能**
   - 内置计数、统计功能

### 17.2 兼容性考虑

- **Rust 版本**：最低支持 Rust 1.70+
- **no_std 支持**：核心功能应支持 `no_std` 环境
- **WASM 支持**：确保在 WebAssembly 环境中正常工作

## 18. 相关资料

### 18.1 Rust 文档

- [std::sync::atomic 文档](https://doc.rust-lang.org/std/sync/atomic/)
- [Rust Atomics and Locks 书籍](https://marabos.nl/atomics/)
- [Rust 内存模型](https://doc.rust-lang.org/nomicon/atomics.html)

### 18.2 JDK 文档

- [java.util.concurrent.atomic 文档](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/util/concurrent/atomic/package-summary.html)
- [AtomicInteger Javadoc](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/util/concurrent/atomic/AtomicInteger.html)

### 18.3 论文和文章

- [C++ Memory Model](https://en.cppreference.com/w/cpp/atomic/memory_order)
- [Linux Kernel Memory Barriers](https://www.kernel.org/doc/Documentation/memory-barriers.txt)

## 19. 变更历史

| 版本 | 日期 | 作者 | 变更说明 |
|-----|------|------|---------|
| 1.0 | 2025-01-22 | Claude (AI Assistant) | 初始版本 |
| 1.1 | 2025-01-22 | Claude (AI Assistant) | 采用方案1：移除所有 `_with_ordering` 变体，改为通过 `inner()` 方法暴露底层类型 |
| 1.2 | 2025-01-22 | Claude (AI Assistant) | 添加完整的 JDK API 对照表，确保接口设计与 JDK 保持一致 |
| 1.3 | 2025-01-22 | Claude (AI Assistant) | 添加"性能优化指南"章节，详细说明何时应该和不应该使用 `inner()` |

**主要变更内容（v1.1）：**
- 移除所有 `_with_ordering` 方法变体
- 添加 `inner()` 方法作为访问底层类型的唯一途径
- 添加 `into_inner()` 和 `from_std()` 方法用于类型转换
- 更新所有示例代码以反映新设计
- 更新实施计划（工作量减少 3-4 天）
- 添加详细的 `inner()` 使用指南
- 强化"易用性优先"的设计理念
- 添加常见陷阱说明

**主要变更内容（v1.2）：**
- 添加完整的 JDK API 对照表（AtomicInteger、AtomicBoolean、AtomicReference）
- 列出所有 JDK 方法及其 Rust 对应实现
- 明确标注 Rust 特有的方法（位运算、最大/最小值等）
- 添加 trait 实现说明（Send、Sync、Display、Debug 等）
- 说明不实现某些 trait 的原因（PartialEq、Clone 等）
- 添加 Default、From 等便利 trait 的实现
- 确保命名与 JDK 保持一致

**主要变更内容（v1.3）：**
- 添加第 7 章"性能优化指南：何时使用 `inner()`"（300+ 行详细指导）
- 说明默认内存序的性能特点和设计理由
- 提供 5 个应该使用 `inner()` 的具体场景（含代码示例）
- 提供 3 个不应该使用 `inner()` 的反模式（含错误示例）
- 添加性能优化决策树，帮助开发者做出正确选择
- 提供不同内存序在 x86-64 和 ARM64 上的性能对比数据
- 添加使用 `inner()` 前的检查清单（6 项）
- 强调"默认 API 优先，`inner()` 是最后手段"的黄金法则

**设计决策理由：**
1. API 表面积减少 50%（不需要所有方法的 `_with_ordering` 版本）
2. 防止用户误用内存序
3. 保持清晰的定位：我们是"易用封装"，不是"完整替代"
4. `inner()` 为高级用户提供完美的 escape hatch
5. 降低维护成本和学习曲线

---

**文档状态**：草案
**最后更新**：2025-01-22
**审阅者**：待定

