# Plan to Rewrite Backend in Gleam

This document outlines the strategy for rewriting the "Project Babel" backend from Rust to **Gleam**, leveraging the **Erlang (BEAM)** runtime.

The BEAM is an ideal fit for this project. The current Rust architecture (using `Arc<Mutex<ChatRoom>>`, channels, and separate room tasks) effectively mimics the **Actor Model**. Gleam/Erlang implements this model natively, likely resulting in simpler, more robust, and fault-tolerant code for "Room" management.

## 1. Technology Stack

| Component | Rust (Current) | Gleam (Proposed) | Notes |
| :--- | :--- | :--- | :--- |
| **Runtime** | Tokio (Async Runtime) | **BEAM (Erlang VM)** | Native fault tolerance & massive concurrency. |
| **Web Server** | Axum | **Mist** (or **Wisp**) | Mist for WebSockets; Wisp for higher-level HTTP routing. |
| **Concurrency** | `tokio::spawn`, `mpsc`, `Mutex` | **`gleam_otp`** | Native implementation of Actors (GenServer). |
| **State** | `Arc<DashMap>` | **Process Registry** | Actors hold their own state; addressed by PID/Subject. |
| **JSON** | Serde (Derived) | **`gleam_json`** | Explicit encoders/decoders required (no reflection). |

---

## 2. Architecture Migration: The Actor Model

In Rust, the system manually constructs a "Room Runner" loop with channels. In Gleam, **every Room will be an OTP Actor (Process)**.

### A. Core Types (`backend/src/data.rs` -> `src/types.gleam`)
Convert Rust structs to Gleam types. Note that Gleam requires explicit JSON decoders.

**Important**: The frontend expects **snake_case** JSON field names (`sender_id`, `was_censored`, `room_closed`). Use explicit field name strings in encoders:

```gleam
// Types
pub type UserAction {
  SendMessage(content: String)
  SendMessageArray(words: List(String))
  SubmitNotes(notes: Dict(String, List(String)))
  LeaveRoom
}

pub type Message {
  Message(id: Int, sender_id: String, sender_country: String, content: String, timestamp: Int)
}

pub type CensoredMessage {
  CensoredMessage(id: Int, sender_id: String, content: String, was_censored: Bool, timestamp: Int)
}

pub type VictoryState {
  VictoryState(achieved: Bool, player_progress: List(PlayerProgress), unlocked_at: Option(Int))
}

pub type PlayerProgress {
  PlayerProgress(user_id: String, country: String, discovered_count: Int, total_required: Int, completed: Bool)
}

// JSON encoding example (must use snake_case for frontend compatibility)
fn encode_censored_message(msg: CensoredMessage) -> Json {
  json.object([
    #("id", json.int(msg.id)),
    #("sender_id", json.string(msg.sender_id)),
    #("content", json.string(msg.content)),
    #("was_censored", json.bool(msg.was_censored)),
    #("timestamp", json.int(msg.timestamp)),
  ])
}
```

### B. The Room Actor (`backend/src/room.rs` -> `src/actors/room.gleam`)
Instead of a `Mutex` guarding state, the state is passed recursively in the actor loop.

*   **State**: `RoomState` (participants, messages, filter config).
*   **Messages (Subject Protocol)**:
    *   `Join(user_id, country, client_subject)`
    *   `UserAction(user_id, action)`
    *   `Leave(user_id)`
    *   `GetState(reply_subject)`
*   **Logic**:
    *   On `UserAction`: Update state, then loop through all connected `client_subjects` and send them the `RoomUpdate`.
    *   The `CensorshipFilter` logic becomes a pure function called by the actor before broadcasting.

### C. The Room Registry (`backend/src/manager.rs` -> `src/actors/manager.gleam`)
Instead of `DashMap<String, RoomConnector>`, use a **Dynamic Supervisor** or a dedicated Registry Actor.
*   **Function**: Spawns `room` actors on demand.
*   **Lookup**: Maps `RoomId` -> `Subject(RoomMessage)`.

---

## 3. Implementation Plan

### Phase 0: API Contract Documentation
Before writing code, document the exact API contract from the frontend:

1.  **HTTP Endpoints** (from `server.rs`):
    ```
    POST   /api/login                       - Get user token
    GET    /api/rooms                       - List room IDs
    POST   /api/rooms                       - Create room
    GET    /api/rooms/:id/info              - Room word lists
    GET    /api/rooms/:id/connect           - WebSocket (participant)
    GET    /api/rooms/:id/spectate          - WebSocket (spectator, read-only)
    POST   /api/rooms/:id/solve             - Check single answer
    POST   /api/rooms/:id/solve_with_note   - Check merged notes
    POST   /api/rooms/:id/submit_notes      - Update player hypotheses
    ```

2.  **WebSocket Messages** (from `frontend/src/types/websocket.ts`):
    *   Inbound: `UserAction` (`send_message`, `leave_room`)
    *   Outbound: `RoomUpdate` (`room_state`, `new_messages`, `notifications`, `room_closed`, `victory`)

3.  **JSON Field Naming**: All fields use **snake_case** (e.g., `sender_id`, `was_censored`, `room_closed`).

### Phase 1: Project & Logic (Pure Gleam)
1.  `gleam new babel_backend`
2.  **Dependencies**: Add `gleam_otp`, `mist`, `wisp`, `gleam_http`, `gleam_json`, `birl` (for time).
3.  **Domain Logic**:
    *   Port `words.rs` (~40 lines): Implement JSON loader for `words.json`.
    *   Port `filter.rs` (~130 lines): Implement the `censor_message` function.
    *   Port `data.rs` types with JSON encoders/decoders (~150 lines expected).
4.  **Unit Tests**: Port the 4 tests from `filter.rs` to verify behavior parity.

### Phase 2: The Room Actor (OTP)
1.  Define the `Room` actor using `gleam_otp/actor`.
2.  **Room State**:
    ```gleam
    pub type RoomState {
      RoomState(
        room_id: String,
        participants: List(Participant),
        messages: List(Message),
        message_counter: Int,
        filter: CensorshipFilter,
        allowed_words: List(String),
        sender_censor: Bool,
        receiver_censor: Bool,
        shadow_ban: Bool,
        allowed_countries: Set(String),
        player_notes: Dict(String, Dict(String, List(String))),
        victory_achieved: Bool,
        victory_timestamp: Option(Int),
      )
    }
    ```
3.  **Message Protocol**:
    *   `Join(user_id, country, client_subject)`
    *   `UserAction(user_id, action)` - includes `SubmitNotes` for player hypotheses
    *   `Leave(user_id)`
    *   `GetState(reply_subject)`
    *   `GetInfo(reply_subject)` - for `/api/rooms/:id/info`
4.  **Victory System** (port from `room.rs:686-769`):
    *   Track `player_notes` per user per country
    *   `check_victory()`: Validate all players discovered all banned words
    *   `get_player_progress()`: Calculate discovery progress for each player
    *   On victory: broadcast system message, update `allowed_countries` to disable censorship
5.  **Broadcast Mechanism**: Hold `List(Subject(RoomUpdate))` for connected clients.

### Phase 3: Web Server (Mist/Wisp)
1.  **HTTP Routes**:
    *   Use `wisp` for routing (higher-level than raw `mist`).
    *   Implement token generation (16-char random alphanumeric).
    *   Implement all endpoints from Phase 0.
2.  **WebSockets** (`/api/rooms/:id/connect`):
    *   Use `mist.websocket`.
    *   **On Init**: Look up Room Actor, send `Join` with WebSocket's `Subject`.
    *   **On Text**: Decode JSON `UserAction` -> Send to Room Actor.
    *   **On Custom Message (from Room)**: Apply per-client censorship, encode to JSON, send to client.
    *   **On Close**: Send `Leave` to Room Actor.
3.  **Spectator Mode** (`/api/rooms/:id/spectate`):
    *   Read-only WebSocket connection.
    *   Receives `RoomUpdate` but cannot send actions.

### Phase 4: Integration
1.  Dockerize: Use the `erlang:alpine` base image.
2.  Update `docker-compose.yml` to point to the Gleam build.
3.  Verify frontend compatibility with all endpoints.

### Phase 5: Test Porting
Port the 22 unit tests from Rust to Gleam:
*   **filter.rs** (4 tests): Sender/receiver censorship, combined filters.
*   **room.rs** (13 tests): Censorship scenarios, shadow ban, allowed countries, victory conditions, note handling.
*   **server.rs** (5 tests): API endpoint behavior.

Use `gleeunit` for testing. Ensure behavior parity with the Rust implementation.

---

## 4. Key Differences & Challenges

1.  **JSON Boilerplate**: You lose `#[derive(Serialize)]`. You will need to write decoders (using `gleam/dynamic`) and encoders (using `gleam/json`) manually. Expect ~150 lines of encoder/decoder code for all types.

2.  **JSON Field Naming**: The frontend expects **snake_case** field names. Gleam's conventions lean toward camelCase. Use explicit field name strings in all JSON encoders:
    ```gleam
    // Correct: produces {"sender_id": "..."}
    json.object([#("sender_id", json.string(msg.sender_id))])

    // Wrong: would produce {"senderId": "..."}
    ```

3.  **Global State**: Instead of `Lazy<FilterConfig>`, pass the config into the Supervisor start argument, or use Erlang's `persistent_term` for static read-only data like word lists.

4.  **Mutable Data**: There is no mutation. Updating the `RoomState` inside the actor means returning a *new* record with the changes.
    ```gleam
    // Rust
    room.messages.push(msg);
    // Gleam
    actor.continue(State(..state, messages: [msg, ..state.messages]))
    ```

5.  **Victory System Complexity**: The victory logic (~80 lines in Rust) involves:
    *   Tracking per-user hypotheses across all countries
    *   Validating exact matches against banned words
    *   Broadcasting progress to all clients
    This requires careful porting with comprehensive tests.

## 5. Recommended Directory Structure

```text
src/
├── babel.gleam           # Application Entrypoint (Supervisor)
├── types.gleam           # Common types (RoomId, UserAction, Message, etc.)
├── json_codec.gleam      # JSON encoders/decoders for all types
├── logic/
│   ├── filter.gleam      # Censorship logic
│   ├── words.gleam       # Word list loading
│   └── victory.gleam     # Victory condition logic
├── actors/
│   ├── room.gleam        # The Room GenServer
│   └── manager.gleam     # Room Registry/Supervisor
└── web/
    ├── router.gleam      # HTTP handlers (wisp)
    └── socket.gleam      # WebSocket handler (participant + spectator)

test/
├── filter_test.gleam     # Port of filter.rs tests
├── room_test.gleam       # Port of room.rs tests
└── api_test.gleam        # Port of server.rs tests
```

---

## 6. Effort Summary

| Phase | Scope | Estimated Lines |
|-------|-------|-----------------|
| Phase 0 | API Contract Documentation | N/A (documentation) |
| Phase 1 | Pure Logic (types, filter, words, JSON codecs) | ~350 lines |
| Phase 2 | Room Actor with Victory System | ~450 lines |
| Phase 3 | Web Server (HTTP + WebSocket) | ~400 lines |
| Phase 4 | Integration (Docker, compose) | ~50 lines |
| Phase 5 | Test Porting (22 tests) | ~300 lines |
| **Total** | | **~1,550 lines** |

The Gleam version should be slightly smaller than the Rust version (~2,200 lines) due to less boilerplate for concurrency, offset by more explicit JSON handling.
