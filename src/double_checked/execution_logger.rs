/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Execution Logger
//!
//! Logging configuration and helpers for the double-checked lock executor.
//!
//! # Author
//!
//! Haixing Hu

use std::fmt;

/// Logger for double-checked execution (condition unmet, prepare/rollback
/// failures).
///
/// Holds log level and unmet message (previous `LogConfig` surface) plus
/// message prefixes for prepare/rollback errors, and an `enabled` switch.
///
/// # Author
///
/// Haixing Hu
#[derive(Debug, Clone)]
pub struct ExecutionLogger {
    /// When `false`, all logging methods are no-ops.
    pub enabled: bool,

    /// Log level for the condition-unmet message.
    pub level: log::Level,

    /// Message logged when the execution condition is not met.
    pub unmet_message: String,

    /// Prefix for prepare-failure lines (logged at error level), formatted as
    /// `"{prefix}: {error}"`.
    pub prepare_failed_message: String,

    /// Prefix for rollback-failure lines (logged at error level), formatted as
    /// `"{prefix}: {error}"`.
    pub rollback_failed_message: String,
}

impl ExecutionLogger {
    /// Creates a logger with default prepare/rollback prefixes matching the
    /// previous hard-coded messages.
    #[inline]
    pub fn new(level: log::Level, unmet_message: impl Into<String>) -> Self {
        Self {
            enabled: true,
            level,
            unmet_message: unmet_message.into(),
            prepare_failed_message: "Prepare action failed".to_string(),
            rollback_failed_message: "Rollback action failed".to_string(),
        }
    }

    /// Logs the configured unmet message at [`Self::level`] when [`Self::enabled`].
    #[inline]
    pub fn log_unmet_message(&self) {
        if !self.enabled {
            return;
        }
        log::log!(self.level, "{}", self.unmet_message);
    }

    /// Logs a prepare-action failure at error level when [`Self::enabled`].
    #[inline]
    pub fn log_prepare_failed<E: fmt::Display>(&self, err: E) {
        if !self.enabled {
            return;
        }
        log::error!("{}: {}", self.prepare_failed_message, err);
    }

    /// Logs a rollback failure at error level when [`Self::enabled`].
    #[inline]
    pub fn log_rollback_failed<E: fmt::Display>(&self, err: E) {
        if !self.enabled {
            return;
        }
        log::error!("{}: {}", self.rollback_failed_message, err);
    }
}
