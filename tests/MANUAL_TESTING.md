# ChessMate Manual Testing Checklist

This document provides comprehensive manual testing procedures for the ChessMate network multiplayer system.

## Pre-Testing Setup

### Environment Preparation

- [ ] Server is built and ready: `cargo build --bin server --release`
- [ ] Client is built and ready: `cargo build --bin client --release`
- [ ] Database is running (if applicable): `docker-compose up db`
- [ ] Ports are available: 3000 (server), 5432 (database)
- [ ] No firewall blocking connections

### Test Environment Options

**Option 1: Local Scripts**
```bash
./scripts/run_local_dev.sh
```

**Option 2: Manual Setup**
```bash
# Terminal 1: Server
./scripts/run_server.sh

# Terminal 2: Client 1
./scripts/run_client.sh ws://localhost:3000/ws alice

# Terminal 3: Client 2
./scripts/run_client.sh ws://localhost:3000/ws bob
```

**Option 3: Docker**
```bash
docker-compose --profile testing up
```

---

## 1. Connection Tests

### 1.1 Basic Connection

**Test**: Single client can connect to server

- [ ] Start server
- [ ] Start one client
- [ ] Verify client shows "Connected" message
- [ ] Verify server logs show connection
- [ ] Check server doesn't crash

**Expected**: ✅ Client connects successfully

**Actual**: ___________________________________________

---

### 1.2 Multiple Connections

**Test**: Multiple clients can connect simultaneously

- [ ] Start server
- [ ] Start client 1
- [ ] Start client 2
- [ ] Start client 3
- [ ] Verify all show "Connected"
- [ ] Verify server handles all connections

**Expected**: ✅ All clients connect without errors

**Actual**: ___________________________________________

---

### 1.3 Rapid Reconnection

**Test**: Client can reconnect after disconnect

- [ ] Connect client
- [ ] Kill client (Ctrl+C)
- [ ] Restart same client
- [ ] Verify reconnection succeeds
- [ ] Repeat 3 times

**Expected**: ✅ Client reconnects each time

**Actual**: ___________________________________________

---

## 2. Matchmaking Tests

### 2.1 Two Player Matchmaking

**Test**: Two players get matched together

- [ ] Start server
- [ ] Start client 1 (alice)
- [ ] Verify alice joins queue
- [ ] Start client 2 (bob)
- [ ] Verify bob joins queue
- [ ] Verify match is created
- [ ] Both clients receive "Match found" message
- [ ] Game ID is displayed
- [ ] Opponent name is correct
- [ ] Colors are assigned (one White, one Black)

**Expected**: ✅ Players matched within 1 second

**Actual**: ___________________________________________

**Match Details**:
- Game ID: ___________________________________________
- Alice color: ___________________________________________
- Bob color: ___________________________________________

---

### 2.2 Odd Number Matchmaking

**Test**: With 3 players, 2 get matched and 1 waits

- [ ] Start server
- [ ] Start 3 clients
- [ ] Verify 2 players get matched
- [ ] Verify 1 player still waiting
- [ ] Verify waiting player shows "Waiting for opponent"
- [ ] Start 4th client
- [ ] Verify waiting players get matched

**Expected**: ✅ Correct pairing behavior

**Actual**: ___________________________________________

---

### 2.3 Color Assignment

**Test**: Players are randomly assigned White/Black

- [ ] Create 5 matches (10 players total)
- [ ] Record color assignments
- [ ] Verify mix of White and Black

**Expected**: ✅ Roughly 50/50 distribution

**Actual Color Distribution**:
- Alice played as: ___________________________________________
- Bob played as: ___________________________________________
- Carol played as: ___________________________________________
- Dave played as: ___________________________________________
- Eve played as: ___________________________________________

---

## 3. Gameplay Tests

### 3.1 Basic Moves

**Test**: Players can make valid chess moves

**Setup**:
- [ ] Two players matched

**Test Steps**:
- [ ] White player makes opening move (e2-e4)
- [ ] Verify move appears on both boards
- [ ] Verify turn switches to Black
- [ ] Black player makes response (e7-e5)
- [ ] Verify move appears on both boards
- [ ] Verify turn switches to White
- [ ] Make 5 more moves
- [ ] Verify all moves sync correctly

**Expected**: ✅ All moves sync instantly

**Actual**: ___________________________________________

**Move List**:
1. ___________________________________________
2. ___________________________________________
3. ___________________________________________
4. ___________________________________________
5. ___________________________________________

---

### 3.2 Invalid Move Rejection

**Test**: Invalid moves are rejected by server

**Test Cases**:

#### 3.2.1 Illegal Chess Move
- [ ] Try to move pawn backward
- [ ] Verify move rejected
- [ ] Error message displayed
- [ ] Board state unchanged

**Expected**: ❌ "Illegal move" error

**Actual**: ___________________________________________

#### 3.2.2 Wrong Turn
- [ ] Black tries to move on White's turn
- [ ] Verify move rejected
- [ ] Error message: "Not your turn"
- [ ] Turn doesn't change

**Expected**: ❌ "Not your turn" error

**Actual**: ___________________________________________

#### 3.2.3 Invalid Piece Selection
- [ ] Try to move opponent's piece
- [ ] Verify move rejected
- [ ] Appropriate error shown

**Expected**: ❌ Move rejected

**Actual**: ___________________________________________

---

### 3.3 Board State Synchronization

**Test**: Board state remains consistent across clients

- [ ] Make 10 moves alternating between players
- [ ] Take screenshots of both client boards
- [ ] Compare board states
- [ ] Verify pieces in identical positions
- [ ] Verify captured pieces match
- [ ] Verify turn indicator matches

**Expected**: ✅ Boards are identical

**Actual**: ___________________________________________

---

### 3.4 Special Moves

#### 3.4.1 Castling
- [ ] Set up position for kingside castling
- [ ] Execute castling move (O-O)
- [ ] Verify both king and rook move
- [ ] Verify move syncs to opponent

**Expected**: ✅ Castling works correctly

**Actual**: ___________________________________________

#### 3.4.2 En Passant
- [ ] Set up en passant situation
- [ ] Execute en passant capture
- [ ] Verify pawn captured correctly
- [ ] Verify move syncs correctly

**Expected**: ✅ En passant works

**Actual**: ___________________________________________

#### 3.4.3 Pawn Promotion
- [ ] Advance pawn to 8th rank
- [ ] Promote to Queen
- [ ] Verify queen appears on board
- [ ] Verify move syncs correctly

**Expected**: ✅ Promotion works

**Actual**: ___________________________________________

---

## 4. Game End Conditions

### 4.1 Resignation

**Test**: Player can resign from game

- [ ] Start game
- [ ] Make a few moves
- [ ] Player resigns
- [ ] Verify "Game Over" message
- [ ] Verify winner displayed
- [ ] Verify both clients notified
- [ ] Verify game removed from server

**Expected**: ✅ Resignation handled correctly

**Actual**: ___________________________________________

**Winner**: ___________________________________________

---

### 4.2 Checkmate

**Test**: Game ends on checkmate

- [ ] Play Scholar's Mate sequence:
  - e2-e4, e7-e5
  - Bc1-c4, Nb8-c6
  - Qd1-h5, Ng8-f6
  - Qh5xf7# (checkmate)
- [ ] Verify checkmate detected
- [ ] Game over message shown
- [ ] Winner announced
- [ ] Both players notified

**Expected**: ✅ Checkmate detected

**Actual**: ___________________________________________

---

### 4.3 Stalemate

**Test**: Game ends in draw on stalemate

- [ ] Set up stalemate position
- [ ] Make final move causing stalemate
- [ ] Verify draw declared
- [ ] Both players notified
- [ ] Reason: "Stalemate"

**Expected**: ✅ Stalemate detected as draw

**Actual**: ___________________________________________

---

## 5. Disconnection Tests

### 5.1 Graceful Disconnect

**Test**: Player disconnects normally

- [ ] Start game
- [ ] Make a few moves
- [ ] Client 1 quits gracefully (quit command)
- [ ] Verify Client 2 notified
- [ ] Verify game cleaned up on server
- [ ] Verify no memory leaks

**Expected**: ✅ Clean disconnect

**Actual**: ___________________________________________

---

### 5.2 Abrupt Disconnect

**Test**: Player connection drops unexpectedly

- [ ] Start game
- [ ] Make a few moves
- [ ] Kill client process (Ctrl+C or kill -9)
- [ ] Wait 5 seconds
- [ ] Verify server detects disconnect
- [ ] Verify opponent notified
- [ ] Verify game cleanup

**Expected**: ✅ Server handles disconnection

**Actual**: ___________________________________________

**Time to detect**: ___________________________________________

---

### 5.3 Network Interruption

**Test**: Handle temporary network issues

- [ ] Start game on separate machines
- [ ] Temporarily disable network
- [ ] Re-enable network
- [ ] Verify connection restored or timeout handled

**Expected**: ✅ Graceful handling

**Actual**: ___________________________________________

---

## 6. Concurrent Games Tests

### 6.1 Multiple Simultaneous Games

**Test**: Server handles multiple games at once

- [ ] Start 4 pairs of clients (8 total)
- [ ] Verify 4 games created
- [ ] Each pair makes 5 moves
- [ ] Verify no cross-game interference
- [ ] Verify all games independent
- [ ] Check server resource usage

**Expected**: ✅ All games run smoothly

**Actual**: ___________________________________________

**Server CPU**: ___________________________________________
**Server Memory**: ___________________________________________
**Active Games**: ___________________________________________

---

### 6.2 Game Isolation

**Test**: Games don't interfere with each other

- [ ] Start 2 games simultaneously
- [ ] Game 1: Make move
- [ ] Verify move only affects Game 1
- [ ] Game 2: Make move
- [ ] Verify move only affects Game 2
- [ ] Verify boards remain separate

**Expected**: ✅ Complete isolation

**Actual**: ___________________________________________

---

## 7. Performance Tests

### 7.1 Response Time

**Test**: Measure move latency

- [ ] Make 20 moves
- [ ] Record time from move input to board update
- [ ] Calculate average latency

**Expected**: ✅ < 100ms average latency

**Actual Latencies (ms)**:
1-5: ___________________________________________
6-10: ___________________________________________
11-15: ___________________________________________
16-20: ___________________________________________

**Average**: ___________________________________________

---

### 7.2 Server Resource Usage

**Test**: Monitor server performance under load

**Idle State**:
- [ ] CPU: ___________________________________________
- [ ] Memory: ___________________________________________
- [ ] Connections: ___________________________________________

**10 Active Games**:
- [ ] CPU: ___________________________________________
- [ ] Memory: ___________________________________________
- [ ] Connections: ___________________________________________

**20 Active Games**:
- [ ] CPU: ___________________________________________
- [ ] Memory: ___________________________________________
- [ ] Connections: ___________________________________________

**Expected**: ✅ Stable resource usage

**Actual**: ___________________________________________

---

### 7.3 Memory Leak Check

**Test**: Ensure no memory leaks over time

- [ ] Record initial memory: ___________________________________________
- [ ] Run for 1 hour with games starting/stopping
- [ ] Record final memory: ___________________________________________
- [ ] Calculate increase: ___________________________________________

**Expected**: ✅ < 10% increase

**Actual**: ___________________________________________

---

## 8. Error Handling Tests

### 8.1 Invalid JSON

**Test**: Server handles malformed messages

- [ ] Send invalid JSON to server
- [ ] Verify server doesn't crash
- [ ] Verify error logged
- [ ] Connection remains stable

**Expected**: ✅ Graceful error handling

**Actual**: ___________________________________________

---

### 8.2 Unknown Message Type

**Test**: Server handles unknown message types

- [ ] Send message with unknown type
- [ ] Verify appropriate error response
- [ ] Connection remains open

**Expected**: ✅ Error message sent to client

**Actual**: ___________________________________________

---

### 8.3 Database Failure

**Test**: Server handles database unavailability

- [ ] Stop database
- [ ] Try to start game
- [ ] Verify graceful degradation
- [ ] Start database
- [ ] Verify recovery

**Expected**: ✅ Degrades gracefully

**Actual**: ___________________________________________

---

## 9. Security Tests

### 9.1 Player ID Validation

**Test**: Server validates player IDs

- [ ] Try empty player ID
- [ ] Try extremely long player ID
- [ ] Try special characters
- [ ] Try SQL injection attempt

**Expected**: ✅ All invalid IDs rejected

**Actual**: ___________________________________________

---

### 9.2 Rate Limiting (Future)

**Test**: Prevent abuse through rate limiting

- [ ] Send 100 moves in 1 second
- [ ] Verify rate limit triggered
- [ ] Verify appropriate error message

**Expected**: ✅ Rate limit enforced

**Actual**: ___________________________________________

---

## 10. Integration Tests

### 10.1 Docker Deployment

**Test**: System works in Docker

- [ ] Build images: `docker-compose build`
- [ ] Start services: `docker-compose up`
- [ ] Run through basic gameplay
- [ ] Stop services: `docker-compose down`
- [ ] Verify clean shutdown

**Expected**: ✅ Works in Docker

**Actual**: ___________________________________________

---

### 10.2 Cross-Platform

**Test**: Clients on different platforms can play

Platforms Tested:
- [ ] macOS ↔ Linux
- [ ] macOS ↔ Windows
- [ ] Linux ↔ Windows

**Expected**: ✅ Cross-platform compatibility

**Actual**: ___________________________________________

---

## Test Summary

### Overall Results

**Date**: ___________________________________________
**Tester**: ___________________________________________
**Environment**: ___________________________________________

**Tests Passed**: ______ / ______
**Tests Failed**: ______
**Tests Skipped**: ______

**Pass Rate**: ______%

### Critical Issues Found

1. ___________________________________________
2. ___________________________________________
3. ___________________________________________

### Minor Issues Found

1. ___________________________________________
2. ___________________________________________
3. ___________________________________________

### Recommendations

1. ___________________________________________
2. ___________________________________________
3. ___________________________________________

### Sign-Off

- [ ] All critical tests passed
- [ ] Known issues documented
- [ ] Ready for deployment

**Tester Signature**: ___________________________________________

**Date**: ___________________________________________

---

## Notes

Use this space for any additional observations or comments:

___________________________________________
___________________________________________
___________________________________________
___________________________________________
___________________________________________
