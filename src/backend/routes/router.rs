/**
 * Router Configuration
 * 
 * This module provides the main router creation function that combines
 * all route configurations into a single Axum router.
 * 
 * # Route Order
 * 
 * Routes are added in a specific order to ensure proper matching:
 * 1. Chat routes (Braid protocol, typing indicators)
 * 2. API routes (auth, usage)
 * 3. Leptos SSR routes (frontend pages)
 * 4. Fallback handler (static files, 404)
 * 
 * # Route Priority
 * 
 * Custom routes are added before Leptos routes to ensure they take
 * precedence. The `/chat` route is specially handled to support both
 * Braid subscriptions and page rendering.
 */

use axum::Router;
#[cfg(feature = "ssr")]
use crate::backend::server::state::AppState;
#[cfg(feature = "ssr")]
// use crate::backend::routes::chat_routes::configure_chat_routes; // not used currently
#[cfg(feature = "ssr")]
use crate::backend::routes::api_routes::configure_api_routes;
use tower_http::services::ServeDir;

/// Create the Axum router with all routes configured
///
/// This function sets up all HTTP routes for the application in the
/// following order:
///
/// 1. **Chat Routes**: Braid protocol endpoints, typing indicators
/// 2. **API Routes**: Authentication, usage statistics
/// 3. **Static Files**: Serve static assets
/// 4. **Fallback Handler**: 404 errors
///
/// # Arguments
///
/// * `app_state` - Application state containing chat state and services
///
/// # Returns
///
/// Configured Axum Router ready to serve requests
///
/// # Route Details
///
/// ## Chat Routes
///
/// - `GET /chat` - Braid subscription
/// - `PUT /chat` - Braid PUT for adding messages
/// - `POST /typing` - Typing indicator events
/// - `GET /realtime` - Generic real-time event subscription
///
/// ## API Routes
///
/// - `POST /api/auth/signup` - User registration
/// - `POST /api/auth/login` - User login
/// - `GET /api/auth/me` - Get current user
/// - `GET /api/usage` - Usage statistics
///
/// ## Static Files
///
/// Static files are served from the public directory.
///
/// ## Fallback
///
/// The fallback handler returns 404 for unknown routes.
#[cfg(feature = "ssr")]
pub fn create_router(app_state: AppState) -> Router<()> {
    // Start with chat routes
    let router = Router::new()
        .route(
            "/chat",
            axum::routing::get({
                use crate::backend::chat::handlers::handle_braid_subscription;
                handle_braid_subscription
            })
            .put({
                use crate::backend::chat::handlers::handle_braid_put;
                handle_braid_put
            }),
        )
        .route(
            "/realtime",
            axum::routing::get({
                use crate::backend::realtime::subscription::handle_realtime_subscription;
                handle_realtime_subscription
            }),
        )
        .route(
            "/typing",
            axum::routing::post({
                use crate::backend::chat::handlers::handle_typing_event;
                handle_typing_event
            }),
        )
        // Collaborative editing routes
        .route(
            "/collab/{doc_id}",
            axum::routing::get({
                use crate::backend::collab::handlers::handle_collab_subscription;
                handle_collab_subscription
            })
            .put({
                use crate::backend::collab::handlers::handle_collab_put;
                handle_collab_put
            }),
        );

    // Add API routes
    let router = configure_api_routes(router);

    // Add static file serving
    let router = router.nest_service("/static", ServeDir::new("public"));

    // Fallback handler for 404
    let router = router.fallback(|| async { "404 Not Found" });

    // Use AppState as router state
    router.with_state(app_state)
}

