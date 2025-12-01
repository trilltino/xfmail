/**
 * XFCollab Server Entry Point
 *
 * This is the main entry point for the XFCollab backend server.
 * It initializes the Axum HTTP server with Braid protocol support.
 */


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Initialize tracing with DEBUG level by default
    let env_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "debug".to_string());
    
    eprintln!("[STARTUP] Setting RUST_LOG={}", env_filter);
    
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&env_filter))
        .with_max_level(tracing::Level::DEBUG)
        .init();

    eprintln!("[STARTUP] Tracing initialized");
    tracing::warn!("[STARTUP] Server initialization started");

    // Create the Axum app
    let app = xfmail::backend::server::init::create_app().await;

    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    eprintln!("[STARTUP] Starting server on {}", addr);
    tracing::warn!("Starting server on {}", addr);

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("[STARTUP] Listening on {}", addr);
    eprintln!("[STARTUP] Client should connect to http://127.0.0.1:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    eprintln!("Server requires the 'ssr' feature to be enabled.");
    eprintln!("Run with: cargo run --bin xfcollab-server --features ssr");
    std::process::exit(1);
}
