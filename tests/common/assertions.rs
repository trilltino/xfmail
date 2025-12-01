//! Custom assertion macros and utilities
//!
//! Provides enhanced assertion macros for better test output and
//! more descriptive error messages.

/// Assert that a result is ok and return the value
///
/// This macro unwraps a Result, providing a better error message
/// if the result is an error.
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
    ($result:expr, $message:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => panic!("{}: {:?}", $message, e),
        }
    };
}

/// Assert that a result is an error
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        assert!($result.is_err(), "Expected Err, got Ok");
    };
    ($result:expr, $pattern:pat) => {
        match $result {
            Err($pattern) => {}
            Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
            Err(e) => panic!("Expected different error variant, got: {:?}", e),
        }
    };
}

/// Assert that two values are approximately equal (for floating point)
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = ($left - $right).abs();
        assert!(
            diff < $epsilon,
            "Values are not approximately equal: {} vs {} (diff: {})",
            $left,
            $right,
            diff
        );
    };
}

/// Assert that a string contains a substring
#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        assert!(
            $haystack.contains($needle),
            "Expected '{}' to contain '{}'",
            $haystack,
            $needle
        );
    };
}

/// Assert that a value is within a range
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "Value {} is not in range [{}, {}]",
            $value,
            $min,
            $max
        );
    };
}
