// Online multiplayer networking module
// Handles client-server communication and matchmaking

pub mod client;
pub mod server;
pub mod matchmaking;
pub mod types;
pub mod protocol;

// websocket module is deprecated - WebSocket handling is now in src/bin/server.rs using Axum
// pub mod websocket;
