// ChessMate multiplayer server - combines REST API and WebSocket game server
use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use chessmate::networking::matchmaking::WaitingPlayer;
use chessmate::networking::protocol::{ClientMessage, ServerMessage};
use chessmate::networking::server::GameServer;

// Application state
#[derive(Clone)]
struct AppState {
    #[allow(dead_code)] // Will be used for future endpoints
    db: PgPool,
    game_server: GameServer,
}

// REST API handlers
async fn health_check() -> &'static str {
    "ChessMate Server is running"
}

async fn stats(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let active_games = state.game_server.active_game_count().await;
    let matchmaking_players = state.game_server.matchmaking_count().await;

    axum::Json(json!({
        "active_games": active_games,
        "matchmaking_players": matchmaking_players,
        "status": "ok"
    }))
}

// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state.game_server))
}

async fn handle_websocket(socket: axum::extract::ws::WebSocket, server: GameServer) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Create a channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn a task to forward messages from the channel to the WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_tx
                    .send(axum::extract::ws::Message::Text(json))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    // Variable to store player ID once they join matchmaking
    let mut player_id: Option<String> = None;

    // Process incoming messages from the WebSocket
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let axum::extract::ws::Message::Text(text) = msg {
                    // Try to deserialize the message
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => {
                            // Extract player_id from the message if we don't have it yet
                            if player_id.is_none() {
                                if let ClientMessage::JoinMatchmaking { player_id: ref pid } =
                                    client_msg
                                {
                                    player_id = Some(pid.clone());

                                    // Add player to matchmaking queue
                                    let player = WaitingPlayer::new(pid.clone(), tx.clone());
                                    if let Err(e) = server.add_to_matchmaking(player).await {
                                        tracing::error!(
                                            "Failed to add player to matchmaking: {}",
                                            e
                                        );
                                        let _ = tx.send(ServerMessage::error(e));
                                        continue;
                                    }

                                    // Send acknowledgment
                                    let _ = tx.send(ServerMessage::matchmaking_joined());
                                    tracing::info!("Player {} joined matchmaking", pid);
                                }
                            }

                            // Handle the message
                            if let Some(ref pid) = player_id {
                                if let Err(e) = server.handle_message(pid, client_msg).await {
                                    tracing::error!("Error handling message from {}: {}", pid, e);
                                    let _ = tx.send(ServerMessage::error(e));
                                }
                            } else {
                                let _ = tx.send(ServerMessage::error(
                                    "Must join matchmaking first".to_string(),
                                ));
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to deserialize message: {}", e);
                            let _ = tx.send(ServerMessage::error(format!(
                                "Invalid message format: {}",
                                e
                            )));
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Client disconnected
    if let Some(pid) = player_id {
        tracing::info!("Player {} disconnected", pid);
    }
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

// Background matchmaking task
async fn matchmaking_loop(server: GameServer) {
    loop {
        // Try to create matches every 500ms
        sleep(Duration::from_millis(500)).await;

        let matches = server.try_matchmaking().await;

        for m in matches {
            tracing::info!(
                "Match created: {} vs {} (Game ID: {})",
                m.white_player.player_id,
                m.black_player.player_id,
                m.game_id
            );

            server.create_game_from_match(m).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting ChessMate multiplayer server...");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/chessmate".to_string());

    // Initialize database
    tracing::info!("Connecting to database...");
    let db_pool = init_database(&database_url).await?;
    tracing::info!("✓ Database connected and migrations applied");

    // Initialize game server
    let game_server = GameServer::new();
    tracing::info!("✓ Game server initialized");

    // Start matchmaking background task
    let matchmaking_server = game_server.clone_refs();
    tokio::spawn(async move {
        matchmaking_loop(matchmaking_server).await;
    });
    tracing::info!("✓ Matchmaking loop started");

    // Create application state
    let state = AppState {
        db: db_pool,
        game_server: game_server.clone_refs(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(stats))
        .route("/ws", get(websocket_handler))
        .layer(cors)
        .with_state(state);

    tracing::info!("✓ Routes configured:");
    tracing::info!("  - Health:    http://0.0.0.0:3000/health");
    tracing::info!("  - Stats:     http://0.0.0.0:3000/stats");
    tracing::info!("  - WebSocket: ws://0.0.0.0:3000/ws");

    // Start server
    let addr = "0.0.0.0:3000";
    tracing::info!("\n🎮 Server ready! Listening on {}", addr);
    tracing::info!("Press Ctrl+C to stop\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
