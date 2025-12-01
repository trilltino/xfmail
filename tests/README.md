# XFCollab Test Suite

This directory contains comprehensive tests for the XFCollab application, including unit tests, integration tests, end-to-end tests, and property-based tests.

## Overview

The test suite is designed to ensure reliability, correctness, and performance across all components of the XFCollab application. Tests cover:

- **Unit Tests**: Individual functions and modules
- **Integration Tests**: API endpoints, database operations, real-time features
- **E2E Tests**: Full browser automation with Fantoccini
- **Property Tests**: Data structure validation with Proptest

## Test Structure

```
tests/
├── common/              # Shared test utilities
│   ├── database.rs     # Database fixtures and setup
│   ├── auth_helpers.rs # Authentication test helpers
│   ├── mock_server.rs  # Mock server utilities
│   └── assertions.rs   # Custom assertion macros
├── integration/        # Integration tests
│   ├── api/           # API endpoint tests
│   ├── database/      # Database operation tests
│   └── realtime/      # Real-time event tests
├── e2e/               # End-to-end browser tests
├── property/           # Property-based tests (Proptest)
└── lib.rs             # Test library and utilities
```

## Running Tests

### All Tests
```bash
cargo test
# or
test.bat
```

### Unit Tests Only
```bash
cargo test --lib
# or
test-unit.bat
```

### Integration Tests
```bash
cargo test --test '*'
# or
test-integration.bat
```

### E2E Tests
```bash
cargo test --test e2e
# or
test-e2e.bat
```

### Coverage Report
```bash
cargo llvm-cov --all-features --workspace --html
# or
test-coverage.bat
```

## Test Coverage Goals

- Minimum 80% line coverage
- Minimum 70% branch coverage
- 100% coverage for critical paths (auth, payments)

## Test Utilities

The `tests/common/` module provides utilities for:
- Database setup and teardown
- Creating test users and tokens
- Mock server configuration
- Custom assertions

## Writing New Tests

### Unit Tests
Add `#[cfg(test)] mod tests` to your source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test code
    }
}
```

### Integration Tests
Create a file in `tests/integration/`:

```rust
#[cfg(feature = "ssr")]
mod tests {
    use tests::common::database::TestDatabase;

    #[tokio::test]
    async fn test_my_integration() {
        let db = TestDatabase::new().await;
        // Test code
    }
}
```

## Requirements

- PostgreSQL database (for integration tests)
- Chrome/Chromium and chromedriver (for E2E tests)
- cargo-llvm-cov (for coverage reports)

