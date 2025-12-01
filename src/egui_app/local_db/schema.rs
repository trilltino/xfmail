//! Database Schema Definitions
//!
//! Contains schema-related constants and utilities.

/// Current database schema version
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Schema migration versions
pub const MIGRATION_VERSIONS: &[i32] = &[1];

/// Check if database needs migration
pub fn needs_migration(current_version: i32) -> bool {
    current_version < CURRENT_SCHEMA_VERSION
}

/// Get pending migrations
pub fn get_pending_migrations(current_version: i32) -> Vec<i32> {
    MIGRATION_VERSIONS
        .iter()
        .filter(|&&v| v > current_version)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
        assert!(!needs_migration(CURRENT_SCHEMA_VERSION));
        assert!(needs_migration(0));
    }

    #[test]
    fn test_pending_migrations() {
        assert_eq!(get_pending_migrations(0), vec![1]);
        assert_eq!(get_pending_migrations(1), Vec::<i32>::new());
    }
}