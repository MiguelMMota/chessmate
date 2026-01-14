# Testing Multiplayer with Godot GUI Clients

This guide explains how to test the multiplayer functionality using two Godot GUI clients.

## Prerequisites

1. **Server running**: Start the ChessMate server
   ```bash
   cargo run --bin chessmate-server
   ```
   The server should be running on `ws://localhost:3000/ws` by default.

2. **Rust library built**: The Godot client needs the compiled Rust library
   ```bash
   cargo build
   ```

## Starting Two Godot Clients

### Option 1: Two Separate Godot Instances

1. **First client**:
   ```bash
   godot --path godot
   ```

2. **Second client** (in a new terminal):
   ```bash
   godot --path godot
   ```

### Option 2: Editor + Running Instance

1. Open Godot Editor:
   ```bash
   godot --path godot --editor
   ```

2. Press F5 to run the game

3. In another terminal, start a second instance:
   ```bash
   godot --path godot
   ```

## Connecting and Playing

### In Each Client:

1. **Set Player Name**: Each client has a randomly generated player ID (e.g., `player_1234`). You can change this in the "Player name" field.

2. **Server URL**: The default `ws://localhost:3000/ws` should work if the server is running locally.

3. **Connect**: Click "Connect & Join Queue" button
   - Status should change to "Connecting..." then "Connected"
   - Then "In matchmaking queue..."

4. **Wait for Match**: Once both clients are in the queue, the server will match them:
   - Status changes to "Match found!"
   - Game info shows:
     - Game ID (shortened)
     - Opponent's name
     - Your color (White or Black)

5. **Play**:
   - White moves first
   - Click and drag pieces (or click source then destination)
   - Only your pieces can be moved on your turn
   - The board updates automatically when your opponent moves

## What to Observe

### Successful Connection:
- Network status shows "Connected" in green
- Then "In matchmaking queue..." in blue
- Finally "Match found!" in green

### During Game:
- Game info panel shows your color and opponent
- Status label shows whose turn it is
- You can only move pieces on your turn
- Board updates after opponent's move
- Invalid moves show error messages

### Error States:
- "Connection failed" - Server not running or wrong URL
- "Invalid move" - Move doesn't follow chess rules
- "Error: ..." - Server-side error

## Debugging

Enable debug output to see detailed logs:
- Check the Godot console output
- Look for messages prefixed with `[DEBUG]`

Common debug messages:
- `Connection established!`
- `Match found - Game ID: ...`
- `Move successful - sending to server!`
- `Game state update from server`

## Testing Scenarios

1. **Basic gameplay**:
   - Start two clients
   - Connect both
   - Play a few moves
   - Verify board stays in sync

2. **Resignation**:
   - Make a few moves
   - One player resigns
   - Check game over message

3. **Disconnect/Reconnect**:
   - Start game
   - Click "Disconnect" on one client
   - Verify the other client receives an error or game over

4. **Invalid moves**:
   - Try illegal moves
   - Verify error messages appear

## Known Limitations

- **Board state sync**: Currently relies on making moves through the game interface. Full board state updates from server are not yet implemented (see `_apply_server_board_state`).
- **Clocks**: Online games don't yet sync clocks with the server
- **Reconnection**: Disconnecting mid-game doesn't support rejoining

## Local vs Online Mode

The client supports both modes:

- **Local mode** (default):
  - Play against yourself or enable "AI plays Black"
  - Clock presets work
  - No network required

- **Online mode** (when connected):
  - Play against remote opponent
  - Your turn is enforced
  - Board updates from server
  - Clock presets disabled in online games
