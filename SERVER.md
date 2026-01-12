# ChessMate Server

The ChessMate server is a REST API that provides game services including user authentication, match history, and card definitions.

## Architecture

The server is built as a separate Rust binary (`chessmate-server`) that runs independently from the game client. It provides:

- **Card Definitions API**: Single source of truth for all card data (name, description, cost, type, set)
- **User Management**: Authentication (email/password and OAuth), user profiles, ratings
- **Match History**: Record of games played, players, winners
- **Card Ownership**: Track which cards each user has unlocked

### Card System

Card definitions are stored in the server code (not the database). Clients fetch card data via the API on startup, ensuring they always have the latest card information. This approach:

- Eliminates data duplication and sync issues
- Allows server-side card updates without client patches
- Maintains single source of truth for game logic
- Prepares for future modding support

## Prerequisites

### PostgreSQL

Install PostgreSQL (version 12 or higher):

**macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15
```

**Ubuntu/Debian:**
```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:**
Download from [postgresql.org](https://www.postgresql.org/download/windows/)

### Create Database

```bash
# Connect to PostgreSQL
psql postgres

# Create database and user
CREATE DATABASE chessmate;
CREATE USER chessmate_user WITH ENCRYPTED PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE chessmate TO chessmate_user;
\q
```

## Configuration

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` and set your database connection:

```
DATABASE_URL=postgres://chessmate_user:your_password@localhost/chessmate
```

## Building

Build the server binary:

```bash
cargo build --bin chessmate-server
```

For production (optimized):

```bash
cargo build --bin chessmate-server --release
```

## Running

### Development

```bash
cargo run --bin chessmate-server
```

The server will:
1. Connect to the database
2. Run migrations automatically
3. Start listening on `http://0.0.0.0:3000`

### Production

```bash
./target/release/chessmate-server
```

Or with custom environment:

```bash
DATABASE_URL=postgres://user:pass@host/db ./target/release/chessmate-server
```

## API Endpoints

### Health Check

```bash
GET /health
```

Returns server status.

**Example:**
```bash
curl http://localhost:3000/health
```

**Response:**
```
ChessMate Server is running
```

### Get Card Definitions

```bash
GET /api/cards
```

Returns all card definitions (single source of truth).

**Example:**
```bash
curl http://localhost:3000/api/cards
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Royal Sacrifice",
    "description": "Remove your queen from the board and gain 5 mana",
    "cost": 3,
    "set": "Basic",
    "card_type": "Instant"
  },
  {
    "id": 2,
    "name": "Pawn Storm",
    "description": "Move all your pawns forward one square",
    "cost": 2,
    "set": "Basic",
    "card_type": "Instant"
  },
  ...
]
```

**Usage in Clients:**

Clients should fetch this endpoint on startup and cache the results locally. This ensures clients always have the latest card data without needing to update client code when cards change.

```gdscript
# Example GDScript code
func load_cards():
    var http = HTTPRequest.new()
    add_child(http)
    http.request("http://localhost:3000/api/cards")
    var response = await http.request_completed
    var cards = JSON.parse_string(response[3].get_string_from_utf8())
    # Cache cards locally...
```

## Database Schema

The server uses PostgreSQL with the following tables:

### `users`
- User accounts, authentication, ratings
- Supports both email/password and OAuth (Google, Apple, etc.)

### `matches`
- Match records with start/end timestamps

### `players`
- Links users to matches with their role (WHITE/BLACK)
- Unique constraint: each user plays once per match

### `match_winners`
- Junction table for match winners
- Supports multiple winners (draws, future team modes)

### `user_owned_cards`
- Tracks which cards each user has unlocked
- `card_id` references card definitions in server code (not a database table)

**Note:** Card definitions are NOT stored in the database. They exist in server code and are served via API.

## Migrations

Migrations are located in `migrations/` and run automatically on server startup.

To manually manage migrations:

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Create new migration
sqlx migrate add migration_name
```

## Development Workflow

1. **Make changes** to server code in `src/bin/server.rs` or migrations
2. **Build** the server: `cargo build --bin chessmate-server`
3. **Run** the server: `cargo run --bin chessmate-server`
4. **Test** endpoints using curl or a REST client

## Testing

Currently, the server doesn't have automated tests. To test manually:

```bash
# Start the server
cargo run --bin chessmate-server

# In another terminal:
curl http://localhost:3000/health
curl http://localhost:3000/api/cards
```

## Future Enhancements

- **Authentication endpoints**: Register, login, OAuth integration
- **Match endpoints**: Create matches, record results, view history
- **User endpoints**: Get profile, update rating, view stats
- **Card unlocking**: Endpoint to unlock cards for users
- **WebSocket support**: Real-time multiplayer game communication
- **Rate limiting**: Prevent API abuse
- **API authentication**: JWT tokens or API keys
- **Automated testing**: Integration tests for all endpoints
