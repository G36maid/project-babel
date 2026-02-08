# 🦀 Project Babel - Backend

**Framework:** Rust (Axum)
**Realtime:** Socketioxide (WebSocket)
**Database:** In-memory (DashMap)

## 🚀 Quick Start (Recommended)

The most recommended way to run the backend is via Docker Compose from the root directory:

```bash
docker-compose up -d backend
```

## 🛠️ Local Development

If you prefer to run it locally without Docker:

1. **Install Rust** (via rustup)
2. **Run Server**
   ```bash
   cargo run
   ```
3. **Run Tests**
   ```bash
   cargo test
   ```

## 📂 Structure

- `src/main.rs`: Entry point and server initialization.
- `src/server.rs`: API routes and WebSocket handlers.
- `src/room.rs`: Chat room logic and state management.
- `src/game.rs`: Game-specific logic and rules.
- `src/filter.rs`: Censorship filtering engine.
- `src/words.rs`: Word list generation and management.
- `src/data.rs`: Type definitions and constants.
- `src/manager.rs`: Room manager for handling multiple rooms.

## 🎮 Action Architecture

The backend uses a layered action architecture to separate concerns:

### Action Types

1. **`SystemAction`** - Room management actions handled by `ChatRoom`/`RoomManager`:
   - `SendMessage(String)` - Send a single message
   - `SendMessageArray(Vec<String>)` - Send multiple words as a message
   - `LeaveRoom` - Leave the current room

2. **`GameAction`** - Game-specific actions delegated to `GameRules`:
   - `SubmitNotes(HashMap<CountryCode, Vec<String>>)` - Submit player hypotheses about banned words

3. **`UserAction`** - Transport layer envelope that wraps the above:
   - `System(SystemAction)` - Contains a system action
   - `Game(GameAction)` - Contains a game action
   - Legacy variants for backward compatibility (SendMessage, SendMessageArray, SubmitNotes, LeaveRoom)

### Processing Flow

```
Client -> UserAction -> ChatRoom::process_action() -> {
    System(...) -> process_system_action() -> RoomManager logic
    Game(...)   -> process_game_action()   -> GameRules delegation
}
```

This separation makes it easier to:
- Add new game mechanics without modifying room management
- Test system and game logic independently
- Maintain clear boundaries between infrastructure and game logic

## 📄 API Documentation

When the server is running, visit:
`http://localhost:3000/swagger-ui/` (if enabled)
