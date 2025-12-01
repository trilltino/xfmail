//! Debug utilities and helpers
//!
//! This module provides debugging utilities that are only available
//! in debug builds or when the debug feature is enabled.

/// Debug mode feature flag
pub const DEBUG_MODE: bool = cfg!(debug_assertions);

/// Debug assertion macro
///
/// Only asserts in debug builds. In release builds, this is a no-op.
#[macro_export]
macro_rules! debug_assert {
    ($condition:expr) => {
        if cfg!(debug_assertions) {
            assert!($condition);
        }
    };
    ($condition:expr, $message:expr) => {
        if cfg!(debug_assertions) {
            assert!($condition, $message);
        }
    };
}

/// Debug assertion with equality check
#[macro_export]
macro_rules! debug_assert_eq {
    ($left:expr, $right:expr) => {
        if cfg!(debug_assertions) {
            assert_eq!($left, $right);
        }
    };
    ($left:expr, $right:expr, $message:expr) => {
        if cfg!(debug_assertions) {
            assert_eq!($left, $right, $message);
        }
    };
}

/// Debug log macro
///
/// Only logs in debug builds or when debug feature is enabled.
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) || cfg!(feature = "debug") {
            tracing::debug!($($arg)*);
        }
    };
}

/// Debug trace macro
#[macro_export]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) || cfg!(feature = "debug") {
            tracing::trace!($($arg)*);
        }
    };
}

/// Validate state invariant
///
/// Checks a state invariant and logs an error if it fails.
/// Only active in debug builds.
pub fn validate_invariant(condition: bool, message: &str) {
    if DEBUG_MODE && !condition {
        tracing::error!("Invariant violation: {}", message);
        #[cfg(debug_assertions)]
        {
            panic!("Invariant violation: {}", message);
        }
    }
}

/// Debug context for tracking execution flow
pub struct DebugContext {
    pub function: &'static str,
    pub file: &'static str,
    pub line: u32,
}

impl DebugContext {
    pub fn new(function: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            function,
            file,
            line,
        }
    }

    pub fn log_entry(&self) {
        if DEBUG_MODE {
            tracing::debug!(
                "Entering {} at {}:{}",
                self.function,
                self.file,
                self.line
            );
        }
    }

    pub fn log_exit(&self) {
        if DEBUG_MODE {
            tracing::debug!(
                "Exiting {} at {}:{}",
                self.function,
                self.file,
                self.line
            );
        }
    }
}

#[macro_export]
macro_rules! debug_context {
    () => {
        $crate::debug::DebugContext::new(
            function!(),
            file!(),
            line!()
        )
    };
}

