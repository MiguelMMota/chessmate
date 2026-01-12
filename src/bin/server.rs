use axum::{
    routing::get,
    Router,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

// Application state
#[derive(Clone)]
struct AppState {
    #[allow(dead_code)] // Will be used for future endpoints
    db: PgPool,
}

// API handlers
async fn health_check() -> &'static str {
    "ChessMate Server is running"
}

// Database initialization
async fn init_database(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/chessmate".to_string());

    tracing::info!("Connecting to database...");
    let db_pool = init_database(&database_url).await?;
    tracing::info!("Database connected and migrations applied");

    // Create application state
    let state = AppState { db: db_pool };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = "0.0.0.0:3000";
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
