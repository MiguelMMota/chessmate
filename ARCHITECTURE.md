# ChessMate Network Multiplayer Architecture

## Overview

ChessMate's network multiplayer architecture enables players on different networks to play chess together in real-time. The system is designed for scalability, low latency, and future compatibility with Web/WebAssembly versions.

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Design Decisions](#design-decisions)
3. [Component Design](#component-design)
4. [Protocol Specification](#protocol-specification)
5. [Data Flow](#data-flow)
6. [Scalability Strategy](#scalability-strategy)
7. [Security Considerations](#security-considerations)
8. [Future Enhancements](#future-enhancements)

## High-Level Architecture

```
┌─────────────────┐         ┌─────────────────┐
│   Godot Client  │         │   CLI Client    │
│   (Future)      │         │   (Testing)     │
└────────┬────────┘         └────────┬────────┘
         │                           │
         │ WebSocket                 │ WebSocket
         │                           │
         └──────────┬────────────────┘
                    │
         ┏━━━━━━━━━━┷━━━━━━━━━━┓
         ┃   Game Server        ┃
         ┃   (Axum + Tokio)    ┃
         ┗━━━━━━━━━┯━━━━━━━━━━┛
                   │
         ┌─────────┴──────────┐
         │   PostgreSQL DB    │
         │   (Future: State)  │
         └────────────────────┘
```

## Design Decisions

### 1. Centralized Server Architecture

**Decision:** Single centralized game server with horizontal scaling capability

**Rationale:**
- Simplifies matchmaking and state synchronization
- Enables authoritative server validation (prevents cheating)
- Easier to reason about and debug than P2P
- Standard architecture for competitive multiplayer games

**Alternatives Considered:**
- **Peer-to-peer (P2P):** Rejected due to NAT traversal complexity, cheating vulnerabilities, and difficulty coordinating state
- **Distributed server mesh:** Rejected as over-engineered for initial version, but architecture supports future migration

### 2. Hybrid Communication Protocol

**Decision:** REST API for matchmaking + WebSocket for gameplay

**Rationale:**
- REST is stateless and simple for queue operations
- WebSocket provides low-latency bidirectional communication for real-time gameplay
- Separating concerns makes each protocol simpler

**Protocol Breakdown:**
- **REST API:** Currently minimal, designed for future expansion (user profiles, leaderboards)
- **WebSocket:** Full-duplex for game state updates and player actions

**Alternatives Considered:**
- **Pure WebSocket:** Rejected because REST is better for stateless operations
- **Server-Sent Events (SSE):** Rejected because we need bidirectional communication
- **gRPC:** Rejected for simplicity, may reconsider for inter-server communication

### 3. WebSocket Architecture

**Decision:** One WebSocket connection per client with game ID routing

**Rationale:**
- Single connection reduces overhead and complexity
- Game ID in messages enables playing multiple games simultaneously (future feature)
- Connection remains open for matchmaking, gameplay, and future notifications

**Implementation:**
- Each WebSocket connection mapped to player ID
- Server maintains `player_to_game` mapping for message routing
- Messages include game ID for multiplexing

**Alternatives Considered:**
- **One WebSocket per game:** Rejected due to connection overhead and complex reconnection logic
- **HTTP polling:** Rejected due to high latency and server load

### 4. FFI Layer Evolution

**Decision:** FFI remains server-side only; clients communicate via WebSocket

**Rationale:**
- Network protocol is cleaner than exposing FFI over network
- FFI is internal implementation detail of server
- Enables non-Rust clients (e.g., JavaScript for web version)

**Architecture:**
```
Client (Any Language)  ──WebSocket──>  Server (Rust)
                                          │
                                          │ FFI (internal)
                                          │
                                          ▼
                                    ChessGame (Rust)
```

### 5. Game State Serialization

**Decision:** Custom `SerializableGameState` separate from internal `ChessGame`

**Rationale:**
- Internal state uses raw pointers and FFI types (not serializable)
- Network state needs to be language-agnostic (JSON)
- Separation allows internal optimizations without breaking protocol

**Trade-offs:**
- Requires conversion between representations (small overhead)
- Keeps protocol clean and stable

### 6. Matchmaking Design

**Decision:** Simple FIFO queue with random color assignment

**Rationale:**
- Simplest approach for MVP
- Fair for casual play
- Foundation for future rating-based matching

**Future Enhancements:**
- ELO/rating-based matchmaking
- Time control preferences
- Friend matching

## Component Design

### Server Components

#### 1. GameServer (`src/networking/server.rs`)

Core game orchestrator managing active games and message routing.

**Responsibilities:**
- Maintain active games map: `HashMap<GameID, ServerGame>`
- Route messages between players
- Validate game actions
- Broadcast state updates
- Handle player disconnections

**Concurrency Model:**
- `Arc<RwLock<HashMap>>` for shared state
- Read-heavy workload optimized with RwLock
- Write locks held briefly during mutations

#### 2. MatchmakingQueue (`src/networking/matchmaking.rs`)

FIFO queue for pairing players.

**Responsibilities:**
- Queue waiting players with timestamps
- Pair players when 2+ are waiting
- Randomly assign colors
- Return matches for game creation

**Thread Safety:**
- Protected by `Arc<RwLock>`
- Background task polls every 500ms

#### 3. WebSocket Handler (`src/bin/server.rs`)

Manages WebSocket lifecycle and message serialization.

**Responsibilities:**
- Upgrade HTTP connections to WebSocket
- Deserialize `ClientMessage` from JSON
- Serialize `ServerMessage` to JSON
- Maintain mpsc channels for async communication
- Handle connection errors and cleanup

#### 4. SerializableGameState (`src/networking/types.rs`)

Network-safe representation of game state.

**Fields:**
```rust
pub struct SerializableGameState {
    pub game_id: String,
    pub current_turn: Color,
    pub status: GameStatus,
    pub white_time: Option<i32>,
    pub black_time: Option<i32>,
    pub board_state: Vec<Vec<Option<Piece>>>,  // 8x8 board
}
```

### Client Components

#### 1. NetworkClient (`src/networking/client.rs`)

Low-level WebSocket client library.

**Responsibilities:**
- Establish WebSocket connection
- Send `ClientMessage` via mpsc channel
- Receive `ServerMessage` via mpsc channel
- Spawn async tasks for send/receive loops

#### 2. SimpleGameClient (`src/networking/client.rs`)

High-level game client with state management.

**Responsibilities:**
- Track current game ID and state
- Process server messages into human-readable events
- Provide convenience methods (submit_move, resign)
- Expose game state to UI layer

#### 3. CLI Client (`src/bin/client.rs`)

Terminal-based test client.

**Features:**
- ASCII board rendering with Unicode pieces
- Non-blocking update loop
- Simple command interface (future: move input)

## Protocol Specification

### Message Format

All messages are JSON-encoded with a `type` discriminator field.

### Client → Server Messages

```typescript
type ClientMessage =
  | { type: "JoinMatchmaking"; player_id: string }
  | { type: "SubmitAction"; game_id: string; action: GameAction }
  | { type: "LeaveGame"; game_id: string }
  | { type: "RequestState"; game_id: string }

type GameAction =
  | { action_type: "MovePiece"; from: Position; to: Position; promotion?: PieceType }
  | { action_type: "Resign" }
  | { action_type: "OfferDraw" }
  | { action_type: "AcceptDraw" }
  | { action_type: "DeclineDraw" }
```

### Server → Client Messages

```typescript
type ServerMessage =
  | { type: "MatchmakingJoined" }
  | { type: "MatchFound"; game_id: string; opponent_id: string; your_color: Color }
  | { type: "GameStateUpdate"; state: SerializableGameState }
  | { type: "OpponentAction"; action: GameAction }
  | { type: "GameOver"; winner?: Color; reason: string }
  | { type: "InvalidAction"; reason: string }
  | { type: "Error"; message: string }
```

### Message Flow Examples

#### Successful Matchmaking

```
Client A → Server: JoinMatchmaking { player_id: "alice" }
Server → Client A: MatchmakingJoined

Client B → Server: JoinMatchmaking { player_id: "bob" }
Server → Client B: MatchmakingJoined

[Background matchmaking task pairs them]

Server → Client A: MatchFound { game_id: "game123", opponent_id: "bob", your_color: White }
Server → Client B: MatchFound { game_id: "game123", opponent_id: "alice", your_color: Black }

Server → Both: GameStateUpdate { state: <initial board> }
```

#### Making a Move

```
Client A → Server: SubmitAction {
  game_id: "game123",
  action: MovePiece { from: (6,4), to: (4,4), promotion: null }
}

[Server validates move]

Server → Client A: GameStateUpdate { state: <updated board> }
Server → Client B: OpponentAction { action: MovePiece {...} }
Server → Client B: GameStateUpdate { state: <updated board> }
```

#### Invalid Move

```
Client A → Server: SubmitAction { game_id: "game123", action: <invalid move> }
Server → Client A: InvalidAction { reason: "It's not your turn" }
```

## Data Flow

### Game Creation Flow

1. Two players join matchmaking queue
2. Background task detects 2+ players
3. Creates `Match` with random colors
4. Creates `ServerGame` with new `ChessGame` instance
5. Sends `MatchFound` to both players
6. Sends initial `GameStateUpdate` to both players

### Move Processing Flow

1. Client sends `SubmitAction` with `MovePiece`
2. Server looks up game from `game_id`
3. Server validates it's player's turn
4. Server calls `game.select_piece(from)`
5. Server calls `game.try_move_selected(to)` or `try_move_selected_with_promotion(to, piece)`
6. Server checks move success
7. Server creates `SerializableGameState` from `game.board_squares()`
8. Server sends `GameStateUpdate` to both players
9. Server sends `OpponentAction` to opponent

### Disconnection Handling

1. WebSocket detects connection close
2. Server removes player's sender from game
3. Server sends `Error` or `GameOver` to remaining player
4. Server cleans up game from `active_games`
5. Server removes mapping from `player_to_game`

## Scalability Strategy

### Current Architecture (Phase 1)

- Single server instance
- In-memory game state
- Suitable for 100s of concurrent games

### Horizontal Scaling (Future Phase 2)

**Strategy:** Add load balancer with sticky sessions

```
         ┌──────────────┐
         │ Load Balancer│
         │ (Sticky)     │
         └──┬────────┬──┘
            │        │
     ┌──────┘        └──────┐
     │                      │
┌────▼─────┐          ┌────▼─────┐
│ Server 1 │          │ Server 2 │
└────┬─────┘          └────┬─────┘
     │                      │
     └──────────┬───────────┘
                │
         ┌──────▼──────┐
         │ PostgreSQL  │
         └─────────────┘
```

**Implementation:**
- WebSocket connections pinned to server (sticky sessions)
- Database stores game state for recovery
- Each game lives on one server (no cross-server coordination)

### Distributed State (Future Phase 3)

**Strategy:** Add Redis for shared state and cross-server matching

```
     ┌─────────────────────┐
     │   Load Balancer     │
     └──┬──────────┬───────┘
        │          │
    ┌───▼───┐  ┌──▼────┐
    │Server1│  │Server2│
    └───┬───┘  └───┬───┘
        │          │
        └────┬─────┘
             │
     ┌───────▼────────┐
     │  Redis Cluster │  ← Shared state, matchmaking queue
     │  PostgreSQL    │  ← Persistent storage
     └────────────────┘
```

## Security Considerations

### Current Implementation

1. **No Authentication:** Player IDs are self-declared (testing only)
2. **Server Authority:** All move validation server-side
3. **Input Validation:** JSON deserialization fails safely
4. **No Rate Limiting:** Currently not implemented

### Future Security Enhancements

1. **Authentication:**
   - JWT tokens for player identity
   - OAuth integration (Google, GitHub)
   - Session management

2. **Authorization:**
   - Validate player is in game before processing actions
   - Prevent spectator interference

3. **Rate Limiting:**
   - Per-connection message rate limits
   - Per-player action rate limits
   - Prevent DOS attacks

4. **TLS/WSS:**
   - Encrypted WebSocket connections (wss://)
   - Certificate management

5. **Input Validation:**
   - Sanitize player IDs and game IDs
   - Validate position ranges
   - Prevent injection attacks

## Future Enhancements

### Short Term

1. **Reconnection Support:**
   - Save game state to database
   - Allow players to reconnect to ongoing games
   - Resume from last known state

2. **Spectator Mode:**
   - Read-only connections to games
   - Broadcast updates to all spectators

3. **Time Controls:**
   - Server-side timer management
   - Countdown clocks for each player
   - Timeout handling

### Medium Term

4. **Rating System:**
   - ELO rating calculation
   - Skill-based matchmaking
   - Leaderboards

5. **Game History:**
   - Store completed games in database
   - PGN export
   - Move-by-move replay

6. **Friend Matching:**
   - Create private games
   - Invite by game code
   - Friends list

### Long Term

7. **Tournament Mode:**
   - Bracket generation
   - Automated scheduling
   - Prize tracking

8. **Analysis Tools:**
   - Post-game analysis
   - Engine evaluation
   - Blunder detection

9. **Mobile/Web Clients:**
   - WebAssembly version
   - Native mobile apps
   - Cross-platform play

## Conclusion

ChessMate's network multiplayer architecture prioritizes simplicity, correctness, and future scalability. The centralized server design with WebSocket communication provides low latency and authoritative validation, while the clean separation between networking and game logic enables easy extension and platform support.

The current implementation serves as a solid foundation for a production-ready multiplayer chess platform, with clear paths for horizontal scaling, feature additions, and security hardening.
