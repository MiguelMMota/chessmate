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

### Build Rust Extension

The project is configured to build for **Apple Silicon (ARM64)** by default on macOS.

Compile the Rust GDExtension library:

```bash
cargo build
```

For optimized release builds:

```bash
cargo build --release
```

The compiled library will be placed in:
- **macOS (ARM64)**: `target/aarch64-apple-darwin/debug/` or `target/aarch64-apple-darwin/release/`
- **macOS (Intel)**: `target/x86_64-apple-darwin/debug/` or `target/x86_64-apple-darwin/release/`
- **Windows**: `target/x86_64-pc-windows-msvc/debug/` or `target/x86_64-pc-windows-msvc/release/`
- **Linux**: `target/x86_64-unknown-linux-gnu/debug/` or `target/x86_64-unknown-linux-gnu/release/`

**Note**: The `.cargo/config.toml` sets the default target to `aarch64-apple-darwin` for Apple Silicon Macs. Update this file if building on a different platform.

### Running in Godot

1. **Open Project in Godot**:
   ```bash
   godot --path . --editor
   ```

2. **Run from Editor**: Press F5 or click the Play button in the Godot editor

3. **Run from Command Line**:
   ```bash
   godot --path .
   ```

### Development Workflow

1. Make changes to Rust code in `src/`
2. Rebuild the extension: `cargo build`
3. Reload the project in Godot (the editor will detect changes)
4. Test changes in the editor or by running the game

## Testing

### Rust Unit Tests

Run Rust unit tests:

```bash
cargo test
```

With verbose output:

```bash
cargo test -- --nocapture
```

### Integration Tests

Run integration tests that involve Godot:

```bash
cargo test --test integration_tests
```

### Game Testing

Test gameplay features directly in Godot:

1. Open the project in Godot editor
2. Navigate to specific test scenes in `scenes/tests/`
3. Run individual scenes to test specific features

## Project Structure

```
chessmate/
├── src/                    # Rust source code
│   ├── lib.rs             # GDExtension entry point
│   ├── game/              # Core game logic
│   ├── ai/                # AI and game evaluation
│   ├── networking/        # Online multiplayer
│   └── cards/             # Card system
├── godot/                 # Godot project files
│   ├── scenes/            # Game scenes
│   ├── scripts/           # GDScript files (if any)
│   ├── assets/            # Art, audio, etc.
│   └── project.godot      # Godot project config
├── tests/                 # Rust integration tests
├── Cargo.toml            # Rust dependencies
└── README.md             # This file
```

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
