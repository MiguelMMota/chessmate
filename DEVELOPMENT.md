# ChessMate Development Guide

This guide covers development workflows, testing strategies, and debugging tips for ChessMate multiplayer.

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Project Architecture](#project-architecture)
3. [Development Workflows](#development-workflows)
4. [Testing Strategies](#testing-strategies)
5. [Debugging Network Issues](#debugging-network-issues)
6. [Common Development Scenarios](#common-development-scenarios)
7. [Code Style & Guidelines](#code-style--guidelines)
8. [Troubleshooting](#troubleshooting)

## Development Environment Setup

### Prerequisites

1. **Rust** (latest stable):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Godot Engine 4.x**:
   - Download from [godotengine.org](https://godotengine.org/download)
   - Add to PATH or note installation location

3. **PostgreSQL** (for network multiplayer):
   ```bash
   # macOS
   brew install postgresql@15
   brew services start postgresql@15

   # Ubuntu/Debian
   sudo apt-get install postgresql-15
   sudo systemctl start postgresql

   # Windows
   # Download installer from postgresql.org
   ```

4. **Docker** (optional, for isolated testing):
   - Download from [docker.com](https://www.docker.com/products/docker-desktop)

### Initial Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd chessmate
   ```

2. **Install Rust dependencies:**
   ```bash
   cargo build
   ```

3. **Set up database:**
   ```bash
   createdb chessmate
   export DATABASE_URL="postgres://postgres:postgres@localhost/chessmate"
   ```

4. **Run tests to verify setup:**
   ```bash
   cargo test
   ```

5. **Test Godot integration:**
   ```bash
   godot --path godot --headless --quit
   ```

### IDE Setup

#### Visual Studio Code

**Recommended extensions:**
- rust-analyzer (rust-lang.rust-analyzer)
- CodeLLDB (vadimcn.vscode-lldb) - for debugging
- Even Better TOML (tamasfe.even-better-toml)
- godot-tools (geequlim.godot-tools)

**Settings (`.vscode/settings.json`):**
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  }
}
```

#### CLion / IntelliJ IDEA

- Install Rust plugin
- Import Cargo project
- Enable Clippy integration

## Project Architecture

### Directory Structure

```
chessmate/
├── src/
│   ├── lib.rs                    # FFI entry point
│   ├── game/                     # Core game logic
│   │   ├── piece.rs             # Chess pieces and positions
│   │   ├── board.rs             # Board state
│   │   └── game_state.rs        # Game management
│   ├── networking/               # Network multiplayer
│   │   ├── protocol.rs          # Message protocol
│   │   ├── types.rs             # Serializable types
│   │   ├── matchmaking.rs       # Matchmaking queue
│   │   ├── server.rs            # Game server logic
│   │   └── client.rs            # Client library
│   └── bin/
│       ├── server.rs            # Server binary
│       └── client.rs            # CLI test client
├── tests/                        # Integration tests
├── godot/                        # Godot project
├── scripts/                      # Helper scripts
├── migrations/                   # Database migrations
└── Cargo.toml                    # Dependencies
```

### Component Responsibilities

**Game Logic (`src/game/`)**
- Pure Rust, no dependencies on networking or Godot
- Handles all chess rules and validation
- Provides clean API for external consumers

**Networking (`src/networking/`)**
- Server-side game orchestration
- WebSocket communication
- Matchmaking and player pairing
- Protocol serialization/deserialization

**Binaries (`src/bin/`)**
- `server.rs`: Standalone game server
- `client.rs`: CLI client for testing

**Godot Integration (`godot/`)**
- Presentation layer only
- Calls into Rust via FFI
- Handles UI, rendering, audio

### Data Flow

```
┌─────────────┐
│ Godot UI    │ ← User input
└─────┬───────┘
      │ FFI calls
      ▼
┌─────────────┐
│ Rust Game   │ ← Game logic & validation
│ Logic       │
└─────┬───────┘
      │ Network protocol
      ▼
┌─────────────┐
│ Game Server │ ← State synchronization
└─────┬───────┘
      │ WebSocket
      ▼
┌─────────────┐
│ Other       │
│ Clients     │
└─────────────┘
```

## Development Workflows

### 1. Working on Game Logic

**Goal**: Modify chess rules, board state, or game mechanics

**Workflow:**
```bash
# 1. Make changes to src/game/
vim src/game/board.rs

# 2. Run unit tests
cargo test game::

# 3. Test integration
cargo test --test integration_tests

# 4. Build and test in Godot
cargo build
godot --path godot --editor
# Press F5 to play
```

**Example: Adding a new game rule**

```rust
// src/game/board.rs

impl Board {
    pub fn is_threefold_repetition(&self) -> bool {
        // Implementation...
    }
}

// tests/integration_tests.rs

#[test]
fn test_threefold_repetition() {
    let mut game = ChessGame::new();
    // ... setup position
    assert!(game.board().is_threefold_repetition());
}
```

### 2. Working on Network Multiplayer

**Goal**: Modify server, client, or protocol

**Workflow:**
```bash
# 1. Make changes to src/networking/
vim src/networking/server.rs

# 2. Run networking tests
cargo test networking::

# 3. Start server
./scripts/run_server.sh

# 4. In separate terminals, start clients
./scripts/run_client.sh ws://localhost:3000/ws alice
./scripts/run_client.sh ws://localhost:3000/ws bob

# 5. Test gameplay
# Clients will auto-match and display the board
```

**Or use the all-in-one script:**
```bash
./scripts/run_local_dev.sh
```

### 3. Working on Godot Integration

**Goal**: Modify UI, rendering, or FFI boundary

**Workflow:**
```bash
# 1. Make changes to Rust FFI (src/lib.rs)
vim src/lib.rs

# 2. Rebuild Rust library
cargo build

# 3. Make changes to Godot scripts
vim godot/chess_board.gd

# 4. Open Godot and test
godot --path godot --editor
# Godot auto-reloads the Rust library
```

### 4. Adding a New Feature

**Full workflow for adding a feature:**

```bash
# 1. Create feature branch
git checkout -b feature/add-draw-offers

# 2. Update protocol (if needed)
vim src/networking/protocol.rs
# Add new message types

# 3. Update game logic
vim src/game/game_state.rs
# Add draw offer tracking

# 4. Update server
vim src/networking/server.rs
# Handle draw offer messages

# 5. Update client
vim src/networking/client.rs
# Send/receive draw offers

# 6. Add tests
vim tests/integration_tests.rs
# Test draw offer flow

# 7. Build and test
cargo build && cargo test

# 8. Test with CLI clients
./scripts/run_local_dev.sh

# 9. Update Godot UI (future)
vim godot/chess_board.gd

# 10. Commit and push
git add .
git commit -m "Add draw offer functionality"
git push origin feature/add-draw-offers
```

## Testing Strategies

### Unit Tests

**Location**: Co-located with source files (`#[cfg(test)] mod tests`)

**Run all unit tests:**
```bash
cargo test
```

**Run specific module:**
```bash
cargo test game::board::
```

**Run with output:**
```bash
cargo test -- --nocapture
```

**Example unit test:**

```rust
// src/networking/matchmaking.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matchmaking_pairs_two_players() {
        let mut queue = MatchmakingQueue::new();

        // Add two players
        queue.add_player(WaitingPlayer { /* ... */ });
        queue.add_player(WaitingPlayer { /* ... */ });

        // Try to create matches
        let matches = queue.try_create_matches();

        assert_eq!(matches.len(), 1);
        assert_eq!(queue.player_count(), 0);
    }
}
```

### Integration Tests

**Location**: `tests/` directory

**Run all integration tests:**
```bash
cargo test --test integration_tests
```

**Example integration test:**

```rust
// tests/integration_tests.rs

use chessmate::game::game_state::ChessGame;
use chessmate::game::piece::{Color, Position};

#[test]
fn test_full_game_flow() {
    let mut game = ChessGame::new();

    // Test opening moves
    game.select_piece(6, 4); // e2
    assert!(game.try_move_selected(4, 4)); // e4

    game.select_piece(1, 4); // e7
    assert!(game.try_move_selected(3, 4)); // e5

    assert_eq!(game.board().current_turn(), Color::White);
}
```

### Network Integration Tests

**Location**: `tests/network_integration_tests.rs`

**Run network tests:**
```bash
cargo test --test network_integration_tests
```

**Example network test:**

```rust
// tests/network_integration_tests.rs

use chessmate::networking::client::SimpleGameClient;
use chessmate::networking::server::GameServer;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_two_clients_match_and_play() {
    // Start server
    let server = GameServer::new();
    tokio::spawn(async move {
        // Run server...
    });

    // Connect two clients
    let mut client1 = SimpleGameClient::new("alice".into(), "ws://localhost:3000/ws".into());
    let mut client2 = SimpleGameClient::new("bob".into(), "ws://localhost:3000/ws".into());

    client1.connect_and_join().await.unwrap();
    client2.connect_and_join().await.unwrap();

    // Wait for matching
    sleep(Duration::from_secs(1)).await;

    // Verify both are in a game
    assert!(client1.in_game());
    assert!(client2.in_game());
}
```

### Manual Testing with CLI

**Test matchmaking:**
```bash
# Terminal 1
./scripts/run_server.sh

# Terminal 2
./scripts/run_client.sh ws://localhost:3000/ws alice

# Terminal 3
./scripts/run_client.sh ws://localhost:3000/ws bob

# Observe: clients auto-match and display board
```

**Test disconnection:**
```bash
# Start server + 2 clients
./scripts/run_local_dev.sh

# Press Ctrl+C to kill one client
# Observe: other client receives game over message
```

### Load Testing

**Using Artillery (future):**

```yaml
# artillery.yml
config:
  target: "ws://localhost:3000"
  phases:
    - duration: 60
      arrivalRate: 10
  engines:
    ws:
      defaults:
        think: 1000

scenarios:
  - engine: ws
    flow:
      - send:
          payload: '{"type":"JoinMatchmaking","player_id":"player_{{ $uuid }}"}'
```

```bash
npm install -g artillery
artillery run artillery.yml
```

## Debugging Network Issues

### Enable Verbose Logging

```bash
export RUST_LOG=debug
./scripts/run_server.sh
```

**Logging levels:**
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: Info, warnings, and errors (default)
- `debug`: Debug info + above
- `trace`: All messages

### Debug WebSocket Messages

**Add logging to server:**

```rust
tracing::debug!("Received message from {}: {:?}", player_id, message);
tracing::debug!("Sending message to {}: {:?}", player_id, response);
```

**Watch real-time logs:**
```bash
tail -f /var/log/chessmate/server.log
```

### Inspect Network Traffic

**Using `tcpdump`:**
```bash
sudo tcpdump -i lo0 port 3000 -A
```

**Using browser DevTools:**
```javascript
// In browser console (for future web client)
const ws = new WebSocket("ws://localhost:3000/ws");
ws.onmessage = (event) => console.log("Received:", event.data);
ws.onopen = () => console.log("Connected");
```

### Common Network Issues

**1. Connection refused:**
```bash
# Check server is running
ps aux | grep server

# Check port is open
lsof -i :3000

# Start server if needed
./scripts/run_server.sh
```

**2. WebSocket upgrade fails:**
```bash
# Check NGINX config (if using reverse proxy)
sudo nginx -t

# Check firewall
sudo ufw status
```

**3. Messages not reaching server:**
```rust
// Add debug logging to client
println!("Sending message: {:?}", message);

// Check serialization
let json = serde_json::to_string(&message)?;
println!("JSON: {}", json);
```

### Debugging Tools

**1. Rust debugging with LLDB:**

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-lldb target/debug/server

# Set breakpoints
(lldb) b server.rs:123
(lldb) run
```

**2. Memory profiling:**

```bash
# Install valgrind (Linux/macOS)
cargo install valgrind

# Profile memory usage
valgrind --leak-check=full ./target/debug/server
```

**3. Performance profiling:**

```bash
# Install flamegraph
cargo install flamegraph

# Profile server
cargo flamegraph --bin server
```

## Common Development Scenarios

### Scenario 1: Adding a New Message Type

**Steps:**

1. Add to protocol:
   ```rust
   // src/networking/protocol.rs

   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "type")]
   pub enum ServerMessage {
       // ... existing variants
       TimeUpdate { white_time: i32, black_time: i32 },
   }
   ```

2. Send from server:
   ```rust
   // src/networking/server.rs

   let msg = ServerMessage::TimeUpdate {
       white_time: game.white_time,
       black_time: game.black_time,
   };
   game.white_sender.send(msg.clone())?;
   game.black_sender.send(msg)?;
   ```

3. Handle in client:
   ```rust
   // src/networking/client.rs

   ServerMessage::TimeUpdate { white_time, black_time } => {
       events.push(format!("Time: White {}s, Black {}s", white_time, black_time));
   }
   ```

4. Test:
   ```bash
   cargo test protocol::
   ./scripts/run_local_dev.sh
   ```

### Scenario 2: Modifying Game Rules

**Steps:**

1. Update game logic:
   ```rust
   // src/game/board.rs

   pub fn is_draw_by_fifty_move_rule(&self) -> bool {
       self.halfmove_clock >= 100
   }
   ```

2. Add test:
   ```rust
   #[test]
   fn test_fifty_move_rule() {
       let mut board = Board::new();
       board.halfmove_clock = 100;
       assert!(board.is_draw_by_fifty_move_rule());
   }
   ```

3. Integrate into game state:
   ```rust
   // src/game/game_state.rs

   pub fn check_game_status(&mut self) {
       if self.board.is_draw_by_fifty_move_rule() {
           self.board.set_status(GameStatus::DrawFiftyMove);
       }
   }
   ```

4. Test and verify:
   ```bash
   cargo test game::
   cargo build
   godot --path godot --editor
   ```

### Scenario 3: Adding Server Configuration

**Steps:**

1. Define config struct:
   ```rust
   // src/networking/server.rs

   #[derive(Debug, Clone)]
   pub struct ServerConfig {
       pub max_games: usize,
       pub matchmaking_interval_ms: u64,
       pub move_timeout_secs: u64,
   }

   impl Default for ServerConfig {
       fn default() -> Self {
           Self {
               max_games: 10000,
               matchmaking_interval_ms: 500,
               move_timeout_secs: 300,
           }
       }
   }
   ```

2. Update server to use config:
   ```rust
   impl GameServer {
       pub fn with_config(config: ServerConfig) -> Self {
           // ...
       }
   }
   ```

3. Load from environment:
   ```rust
   // src/bin/server.rs

   let config = ServerConfig {
       max_games: env::var("MAX_GAMES")
           .ok()
           .and_then(|s| s.parse().ok())
           .unwrap_or(10000),
       // ...
   };
   ```

## Code Style & Guidelines

### Rust Code Style

**Follow Rust conventions:**
```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check before committing
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

**Naming conventions:**
- Types: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### SOLID Principles

**Single Responsibility Principle:**
```rust
// GOOD: Each function has one purpose
pub fn select_piece(&mut self, row: i8, col: i8) { /* ... */ }
pub fn get_valid_moves(&self) -> Vec<Position> { /* ... */ }

// BAD: Function does multiple things
pub fn select_piece_and_move(&mut self, from: Position, to: Position) { /* ... */ }
```

**Dependency Inversion:**
```rust
// GOOD: Depend on traits
pub trait GameClient {
    async fn send_message(&self, msg: ClientMessage) -> Result<()>;
}

// BAD: Depend on concrete types
pub fn handle_game(client: &NetworkClient) { /* ... */ }
```

### Error Handling

**Use `Result` for recoverable errors:**
```rust
pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
    // ...
}
```

**Use custom error types for domain errors:**
```rust
#[derive(Debug)]
pub enum GameError {
    InvalidMove(String),
    NotYourTurn,
    GameOver,
}

impl std::error::Error for GameError {}
```

### Documentation

**Document public APIs:**
```rust
/// Attempts to move the selected piece to the given position.
///
/// # Arguments
///
/// * `row` - The target row (0-7)
/// * `col` - The target column (0-7)
///
/// # Returns
///
/// `true` if the move was successful, `false` otherwise.
///
/// # Examples
///
/// ```
/// let mut game = ChessGame::new();
/// game.select_piece(6, 4); // e2
/// assert!(game.try_move_selected(4, 4)); // e4
/// ```
pub fn try_move_selected(&mut self, row: i8, col: i8) -> bool {
    // ...
}
```

## Troubleshooting

### Build Failures

**"cannot find function in module":**
```bash
# Clean and rebuild
cargo clean
cargo build
```

**"proc-macro derive panicked":**
```bash
# Update serde
cargo update -p serde
cargo build
```

### Test Failures

**"connection refused" in tests:**
```bash
# Ensure no server is running on port 3000
lsof -ti:3000 | xargs kill

# Run tests
cargo test
```

**Timeout in async tests:**
```rust
// Increase timeout
#[tokio::test(flavor = "multi_thread")]
async fn test_slow_operation() {
    tokio::time::timeout(
        Duration::from_secs(10),
        slow_operation()
    ).await.unwrap();
}
```

### Godot Integration Issues

**"Cannot find library" error:**
```bash
# Rebuild Rust library
cargo build

# Check library path in godot/chessmate.gdextension
# Should match your platform:
# - macOS: target/aarch64-apple-darwin/debug/libchessmate.dylib
# - Linux: target/debug/libchessmate.so
# - Windows: target/debug/chessmate.dll
```

**Godot not reloading changes:**
```bash
# Restart Godot Editor
# Or use --quit to force reload
godot --path godot --headless --quit
godot --path godot --editor
```

## Next Steps

- Review [ARCHITECTURE.md](ARCHITECTURE.md) for system design details
- Check [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment
- See [README.md](README.md) for quick start guide

## Contributing

When contributing:
1. Create a feature branch
2. Write tests for new features
3. Ensure all tests pass: `cargo test`
4. Format code: `cargo fmt`
5. Lint code: `cargo clippy`
6. Submit a pull request

For questions or issues, open a GitHub issue or contact the maintainers.
