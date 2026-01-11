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

The project uses a **decoupled architecture** with separate Rust and Godot components:
- **Rust library** (root directory) - Game logic, compiles to a shared library
- **Godot project** (`godot/` directory) - Presentation layer, loads the Rust library via GDExtension

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

## Development Guidelines

- Follow SOLID principles, especially Single Responsibility
- Write unit tests for all game logic
- Document public APIs and complex algorithms
- Use meaningful commit messages
- Ensure builds pass before committing

## License

[To be determined]

## Contributing

[To be determined]
