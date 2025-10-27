/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! # Atomic Types
//!
//! Provides easy-to-use atomic types with sensible default memory orderings.
//! These types wrap `std::sync::atomic` types and provide a more ergonomic
//! API similar to Java's `java.util.concurrent.atomic` package.
//!
//! # Features
//!
//! - Automatic memory ordering selection for common use cases
//! - Rich set of high-level operations (increment, decrement, functional
//!   updates, etc.)
//! - Zero-cost abstraction with inline methods
//! - Access to underlying types via `inner()` for advanced use cases
//!
//! # Author
//!
//! Haixing Hu

#[macro_use]
mod atomic_integer_macro;

mod atomic_bool;
mod atomic_f32;
mod atomic_f64;
mod atomic_i16;
mod atomic_i32;
mod atomic_i64;
mod atomic_i8;
mod atomic_isize;
mod atomic_ref;
mod atomic_u16;
mod atomic_u32;
mod atomic_u64;
mod atomic_u8;
mod atomic_usize;
mod traits;

pub use atomic_bool::AtomicBool;
pub use atomic_f32::AtomicF32;
pub use atomic_f64::AtomicF64;
pub use atomic_i16::AtomicI16;
pub use atomic_i32::AtomicI32;
pub use atomic_i64::AtomicI64;
pub use atomic_i8::AtomicI8;
pub use atomic_isize::AtomicIsize;
pub use atomic_ref::AtomicRef;
pub use atomic_u16::AtomicU16;
pub use atomic_u32::AtomicU32;
pub use atomic_u64::AtomicU64;
pub use atomic_u8::AtomicU8;
pub use atomic_usize::AtomicUsize;
pub use traits::{Atomic, AtomicInteger, UpdatableAtomic};
