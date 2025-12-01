//! Route Configuration Module
//!
//! This module configures all HTTP routes for the backend server.
//! Routes are organized by functionality into focused submodules.
//!
//! # Architecture
//!
//! The routes module is organized into focused submodules:
//!
//! - **`router`** - Main router creation and route assembly
//! - **`chat_routes`** - Chat-related routes (Braid protocol, typing)
//! - **`api_routes`** - API endpoints (auth, usage)
//!
//! # Module Structure
//!
//! ```
//! routes/
//! ├── mod.rs          - Module exports and documentation
//! ├── router.rs       - Main router creation
//! ├── chat_routes.rs  - Chat-specific route handlers
//! └── api_routes.rs   - API endpoint handlers
//! ```
//!
//! # Route Organization
//!
//! Routes are added in a specific order to ensure proper matching:
//!
//! 1. **Chat Routes** - Braid protocol endpoints, typing indicators
//! 2. **API Routes** - Authentication, usage statistics
//! 3. **Leptos SSR Routes** - Frontend page rendering
//! 4. **Fallback Handler** - Static files and 404 errors
//!
//! # Route Types
//!
//! ## Chat Routes
//!
//! - `GET /chat` - Braid subscription or page rendering
//! - `PUT /chat` - Braid PUT for adding messages
//! - `POST /typing` - Typing indicator events
//! - `GET /realtime` - Generic real-time event subscription
//!
//! ## API Routes
//!
//! - `POST /api/auth/signup` - User registration
//! - `POST /api/auth/login` - User login
//! - `GET /api/auth/me` - Get current user
//! - `GET /api/usage` - Usage statistics
//!
//! ## Leptos Routes
//!
//! All other routes are handled by Leptos SSR, including:
//! - `/` - Home page
//! - `/demo` - Demo page
//! - `/login` - Login page
//! - `/signup` - Signup page
//!
//! # Example
//!
//! ```rust,no_run
//! use braid_site::backend::routes::create_router;
//! use braid_site::backend::server::state::AppState;
//! use braid_site::frontend::app::{App, shell};
//!
//! # async fn example() {
//! let app_state = AppState::default();
//! let router = create_router(app_state, App, shell).await;
//! # }
//! ```
//!
//! # Dependencies
//!
//! - `backend::server::state` - Application state
//! - `backend::chat::handlers` - Chat route handlers
//! - `backend::realtime` - Real-time event handlers
//! - `backend::auth` - Authentication handlers
//! - `backend::subscription` - Usage statistics handlers

/// Main router creation
pub mod router;

/// Chat-related route handlers
pub mod chat_routes;

/// API endpoint handlers
pub mod api_routes;

// Re-export commonly used functions
#[cfg(feature = "ssr")]
pub use router::create_router;

