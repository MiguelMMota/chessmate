// Online multiplayer networking module
// Handles client-server communication and matchmaking

pub mod client;
pub mod matchmaking;
pub mod protocol;
pub mod server;
pub mod types;

// websocket module is deprecated - WebSocket handling is now in src/bin/server.rs using Axum
// pub mod websocket;
