//! API integration tests
//!
//! Integration tests for all API endpoints

#[cfg(feature = "ssr")]
mod auth_test;
#[cfg(feature = "ssr")]
mod chat_test;
#[cfg(feature = "ssr")]
mod stripe_test;
#[cfg(feature = "ssr")]
mod subscription_test;
