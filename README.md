# ChessMate

A strategic chess variation game that combines traditional chess mechanics with TCG-style card-based actions, creating a unique blend of tactical depth and unpredictable gameplay.

## Game Description

ChessMate reimagines chess by introducing a card-drafting system where players use cards to control their moves and actions. Unlike traditional chess where all information is complete and deterministic, ChessMate incorporates hidden information and probabilistic elements through its card mechanics, creating a fresh strategic experience.

Players must balance traditional chess tactics with resource management, card drafting, and adapting to incomplete information about their opponent's available actions. The game retains the core strategic depth of chess while adding new layers of decision-making and uncertainty.

## Tech Stack

- **Language**: Rust
- **Game Engine**: Godot 4.x
- **Integration**: GDExtension (Rust bindings for Godot)
- **Build System**: Cargo (Rust) + Godot build pipeline
- **Version Control**: Git/GitHub

## Architecture & Decoupling Requirements

**⚠️ CRITICAL: This project enforces strict architectural separation between server and client code.**

### Core Principles

1. **Rust is Client-Agnostic**
   - The Rust codebase (`src/`) contains ONLY game logic, server code, and core business rules
   - Rust code MUST NOT import, reference, or depend on any client-specific types, frameworks, or APIs
   - Rust code MUST NOT know about Godot, UI frameworks, rendering, or any presentation layer concerns
   - All Rust dependencies in `Cargo.toml` must be platform-agnostic libraries (no GUI, no rendering)

2. **FFI Layer as the Boundary**
   - ALL communication between Rust (server/logic) and clients (Godot, future web clients, etc.) happens through the FFI layer in `src/lib.rs`
   - The FFI layer is a **thin, minimal interface** exposing only essential functions
   - FFI functions should be called **infrequently** and handle **complete operations** (not granular state changes)
   - Data crossing the FFI boundary must use simple, platform-agnostic types (primitives, POD structs, C-compatible representations)

3. **Client Responsibilities**
   - Godot (GDScript) and any future clients are responsible for:
     - Presentation layer (UI, rendering, animations, audio)
     - Input capture and conversion to action data
     - Visual feedback and effects
   - Clients call into Rust for game logic but never the reverse

### What This Means for Development

**✅ DO:**
- Implement game rules, state management, and AI in pure Rust
- Use platform-agnostic Rust libraries (serde, tokio, rand, etc.)
- Expose minimal, batch-oriented functions through FFI (`process_action`, `get_game_state`)
- Keep FFI data structures simple and C-compatible

**❌ DON'T:**
- Add Godot types or dependencies to Rust code
- Import UI frameworks, rendering libraries, or windowing systems in Rust
- Create fine-grained FFI functions that require frequent calls
- Make Rust code aware of how it will be presented or rendered

### Why This Matters

This architecture enables:
- **Portability**: The same Rust logic can power Godot desktop clients, web clients, mobile apps, or headless servers
- **Testability**: Pure Rust logic can be tested without any client dependencies
- **Performance**: Minimal FFI crossing reduces overhead
- **Maintainability**: Clear separation of concerns makes the codebase easier to understand and modify

**If you're adding a new feature, ask yourself: "Could this Rust code run in a headless server with zero knowledge of how the game is displayed?" If the answer is no, you're violating the architecture.**

## Core Features

### Gameplay
- **Chess Variation Mechanics**: Core chess-like gameplay with card-driven action system
- **Card Drafting System**: TCG-inspired card selection and deck-building
- **Chess Clock**: Time controls for competitive play
- **Game State Evaluation**: Advanced engine for evaluating positions with incomplete and non-deterministic information

### Game Modes
- **Singleplayer Mode**: Play against computer AI opponents
- **Online Multiplayer**: Real-time online matches against other players
- **Rank-Based Matchmaking**: Competitive ladder system with skill-based pairing

### Progression
- **Achievements System**: Unlock rewards and track accomplishments
- **Player Statistics**: Detailed tracking of games, win rates, and performance

## Development Setup

### Prerequisites

1. **Rust** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Godot Engine 4.x**
   - Download from [godotengine.org](https://godotengine.org/download)
   - Add Godot to your PATH or note the installation location

3. **GDExtension for Rust**
   ```bash
   cargo install gdext
   ```

4. **tmux** (for multiplayer testing)
   ```bash
   # macOS
   brew install tmux

   # Linux (Ubuntu/Debian)
   sudo apt-get install tmux

   # Linux (Fedora)
   sudo dnf install tmux
   ```

### Initial Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd chessmate
   ```

2. Initialize Rust workspace:
   ```bash
   cargo init
   ```

3. Add GDExtension dependencies to `Cargo.toml`:
   ```toml
   [dependencies]
   godot = { git = "https://github.com/godot-rust/gdext", branch = "master" }
   ```

## Building and Running

The project uses a **strictly decoupled architecture** with separate Rust and Godot components (see [Architecture & Decoupling Requirements](#architecture--decoupling-requirements) for critical guidelines):
- **Rust library** (root directory) - Pure game logic and server code, compiles to a shared library
- **Godot project** (`godot/` directory) - Presentation layer only, loads the Rust library via GDExtension FFI

### Build Rust Library

The Rust library compiles to a shared library (`.dylib` on macOS, `.dll` on Windows, `.so` on Linux) that Godot loads at runtime.

**Build for debug:**
```bash
cargo build
```

**Build for release (optimized):**
```bash
cargo build --release
```

**Platform-specific output locations:**
- **macOS (ARM64)**: `target/aarch64-apple-darwin/debug/libchessmate.dylib`
- **macOS (Intel)**: `target/x86_64-apple-darwin/debug/libchessmate.dylib`
- **Windows**: `target/debug/chessmate.dll`
- **Linux**: `target/debug/libchessmate.so`

**Note**: The `.cargo/config.toml` sets the default target to `aarch64-apple-darwin` for Apple Silicon Macs. Update this file if building on a different platform.

### Run Tests

Run Rust unit tests:
```bash
cargo test
```

Run with test output visible:
```bash
cargo test -- --nocapture
```

### Run the Godot Project

The Godot project is located in the `godot/` subdirectory and automatically loads the compiled Rust library.

**Open in Godot Editor:**
```bash
godot --path godot --editor
```

Or on macOS with Godot in Applications:
```bash
/Applications/Godot.app/Contents/MacOS/Godot --path godot --editor
```

**Run the game (headless, for validation):**
```bash
godot --path godot --headless --quit
```

**Run the game (with GUI):**
```bash
godot --path godot
```

Or from the Godot Editor: Press F5 or click the Play button

### Development Workflow

1. **Make changes** to Rust code in `src/`
2. **Rebuild** the Rust library:
   ```bash
   cargo build
   ```
3. **Test** your changes:
   ```bash
   cargo test
   ```
4. **Reload** in Godot:
   - If Godot Editor is open, it will automatically detect the library changes
   - Or restart the Godot Editor
5. **Run** the game from Godot to test gameplay

### Quick Start Commands

**Build everything and validate:**
```bash
cargo build && godot --path godot --headless --quit
```

**Build and test Rust library:**
```bash
cargo build && cargo test
```

**Full verification (build + tests + Godot validation):**
```bash
cargo build && cargo test && godot --path godot --headless --quit
```

## Testing

### Rust Unit Tests

Test the Rust game logic (runs independently of Godot):

```bash
cargo test
```

With verbose output:
```bash
cargo test -- --nocapture
```

Run a specific test:
```bash
cargo test test_name
```

### Integration Tests

Run integration tests (located in `tests/` directory):
```bash
cargo test --test integration_tests
```

### Game Testing in Godot

Test gameplay and UI directly in the Godot editor:

1. Build the Rust library: `cargo build`
2. Open the Godot project: `godot --path godot --editor`
3. Press F5 to run the game or run specific test scenes from `godot/scenes/tests/`

## Project Structure

```
chessmate/
├── src/                          # Rust source code (game logic)
│   ├── lib.rs                   # GDExtension entry point & FFI layer
│   ├── game/                    # Core game logic
│   ├── ai/                      # AI and game evaluation
│   ├── networking/              # Online multiplayer (server)
│   └── cards/                   # Card system logic
│
├── godot/                       # Godot project (presentation layer)
│   ├── project.godot            # Godot project configuration
│   ├── chessmate.gdextension    # Links to compiled Rust library
│   ├── chess_board.gd           # Main game board (GDScript client)
│   ├── debug_utils.gd           # Debug utilities
│   ├── scenes/                  # Game scenes (.tscn files)
│   ├── scripts/                 # Additional GDScript files
│   └── assets/                  # Art, audio, fonts, etc.
│
├── tests/                       # Rust integration tests
├── scripts/                     # Build and utility scripts
├── .cargo/config.toml          # Cargo build configuration (target platform)
├── .claude/                     # Claude Code commands and config
├── Cargo.toml                   # Rust dependencies and build config
└── README.md                    # This file
```

**Key Files:**
- `src/lib.rs`: FFI boundary between Rust and Godot
- `godot/chessmate.gdextension`: Configuration that tells Godot where to find the compiled Rust library
- `godot/chess_board.gd`: Main Godot script that calls into Rust for game logic
- `Cargo.toml`: Defines the library type (`cdylib` for shared library, `rlib` for Rust tests)

## Network Multiplayer

ChessMate supports real-time multiplayer where players on different networks can play together. The system uses a centralized game server with WebSocket communication for low-latency gameplay.

### Architecture Overview

- **Server**: Rust-based game server with authoritative validation
- **Protocol**: REST API for matchmaking + WebSocket for gameplay
- **Clients**: CLI client for testing, Godot client (future)
- **Database**: PostgreSQL for persistent state (future)

For detailed architecture documentation, see [ARCHITECTURE.md](ARCHITECTURE.md).

### Quick Start: Local Testing

**GUI Clients (Godot):**

Test multiplayer with Docker server + N Godot clients:

```bash
# Launch with 2 GUI clients (default)
./scripts/run_multiplayer.sh

# Launch with 3 GUI clients
./scripts/run_multiplayer.sh 3
```

This script automatically:
- Starts Docker containers (server + database) in a tmux window
- Launches N Godot GUI clients in separate tmux windows
- Keeps all logs visible and attached
- Handles cleanup on exit (Ctrl+C)

**Navigation:** Use `Ctrl+b n` (next window), `Ctrl+b p` (previous window), or `Ctrl+b 0-9` (jump to window)

**CLI Clients (for testing without GUI):**

Test multiplayer locally with server + 2 CLI clients:

```bash
./scripts/run_local_dev.sh
```

This script automatically:
- Builds the server and client binaries
- Starts the game server on port 3000
- Launches 2 CLI clients that auto-connect
- Handles cleanup on exit (Ctrl+C)

### Manual Setup

**1. Start the server:**
```bash
./scripts/run_server.sh
```

The server will listen on `ws://localhost:3000/ws`

**2. Connect clients:**

In separate terminals, run:
```bash
./scripts/run_client.sh ws://localhost:3000/ws player1
./scripts/run_client.sh ws://localhost:3000/ws player2
```

Clients will automatically join the matchmaking queue and be paired when 2+ players are waiting.

### Docker Setup

Run everything in isolated containers:

**Start server and database:**
```bash
docker-compose up
```

**Run with test clients:**
```bash
docker-compose --profile testing up
```

This starts:
- PostgreSQL database
- Game server
- 2 CLI test clients (with `testing` profile)

**Stop and clean up:**
```bash
docker-compose down -v
```

### Environment Variables

Configure the server and clients with these environment variables:

**Server:**
- `DATABASE_URL`: PostgreSQL connection string (default: `postgres://postgres:postgres@localhost/chessmate`)
- `RUST_LOG`: Logging level (default: `info`)

**Client:**
- `SERVER_URL`: WebSocket server URL (default: `ws://localhost:3000/ws`)
- `PLAYER_ID`: Unique player identifier (default: auto-generated)

Example:
```bash
export SERVER_URL="ws://your-server.com:3000/ws"
export PLAYER_ID="alice"
./scripts/run_client.sh
```

### CLI Client Commands

The CLI client displays the board and game state automatically. Future versions will support interactive commands:

- `move <from> <to>` - Make a move (e.g., `move e2 e4`)
- `resign` - Resign from the current game
- `quit` - Disconnect and exit

### Deployment

For production deployment instructions, see [DEPLOYMENT.md](DEPLOYMENT.md).

For development and testing workflows, see [DEVELOPMENT.md](DEVELOPMENT.md).

## Development Guidelines

- **CRITICAL**: Maintain strict architectural decoupling (see [Architecture & Decoupling Requirements](#architecture--decoupling-requirements))
- **Never** add client-specific dependencies to Rust code
- **Always** communicate between Rust and clients through the FFI layer only
- Follow SOLID principles, especially Single Responsibility
- Write unit tests for all game logic (Rust tests should run without any client dependencies)
- Document public APIs and complex algorithms
- Use meaningful commit messages
- Ensure builds pass before committing

## License

[To be determined]

## Contributing

[To be determined]
