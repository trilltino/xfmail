//! Test suite for XFCollab
//!
//! This module organizes all tests

pub mod common;
#[cfg(feature = "ssr")]
pub mod e2e;
pub mod integration;
pub mod property;
