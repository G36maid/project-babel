# Plan to Rewrite Backend in Gleam

This document outlines the strategy for rewriting the "Project Babel" backend from Rust to **Gleam**, leveraging the **Erlang (BEAM)** runtime.

The BEAM is an ideal fit for this project. The current Rust architecture (using `Arc<Mutex<ChatRoom>>`, channels, and separate room tasks) effectively mimics the **Actor Model**. Gleam/Erlang implements this model natively, likely resulting in simpler, more robust, and fault-tolerant code for "Room" management.

## 1. Technology Stack

| Component | Rust (Current) | Gleam (Proposed) | Notes |
| :--- | :--- | :--- | :--- |
| **Runtime** | Tokio (Async Runtime) | **BEAM (Erlang VM)** | Native fault tolerance & massive concurrency. |
| **Web Server** | Axum | **Mist** | Excellent support for WebSockets and HTTP. |
| **Concurrency** | `tokio::spawn`, `mpsc`, `Mutex` | **`gleam_otp`** | Native implementation of Actors (GenServer). |
| **State** | `Arc<DashMap>` | **Process Registry** | Actors hold their own state; addressed by PID/Subject. |
| **JSON** | Serde (Derived) | **`gleam_json`** | Explicit encoders/decoders required (no reflection). |
| **OpenAPI** | `utoipa` (Macros) | *Manual OpenAPI spec* | Tooling is nascent; spec file may need manual maintenance. |

---

## 2. Architecture Migration: The Actor Model

In Rust, the system manually constructs a "Room Runner" loop with channels. In Gleam, **every Room will be an OTP Actor (Process)**.

### A. Core Types (`backend/src/data.rs` -> `src/types.gleam`)
Convert Rust structs to Gleam types. Note that Gleam requires explicit JSON decoders.

```gleam
// Example Translation
pub type UserAction {
  SendMessage(content: String)
  SendMessageArray(words: List(String))
  SendNote(notes: Dict(String, List(String)))
  LeaveRoom
}

pub type Message {
  Message(id: Int, sender_id: String, content: String, ...)
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

### Phase 1: Project & Logic (Pure Gleam)
1.  `gleam new babel_backend`
2.  **Dependencies**: Add `gleam_otp`, `mist`, `gleam_http`, `gleam_json`, `birl` (for time).
3.  **Domain Logic**:
    *   Port `words.rs`: Implement JSON loader for `words.json`.
    *   Port `filter.rs`: Implement the `censor_message` function. String manipulation in Gleam is standard.
    *   **Unit Tests**: Verify the filter logic matches Rust's behavior.

### Phase 2: The Room Actor (OTP)
1.  Define the `Room` actor using `gleam_otp/actor`.
2.  Implement the handle loop.
3.  **Broadcast Mechanism**: The Room Actor must hold a `List(Subject(RoomUpdate))` representing connected WebSocket clients.
4.  Implement `handle_call` for synchronous queries (like `GetInfo`).

### Phase 3: Web Server (Mist)
1.  **HTTP Routes** (`/api/login`, `/api/rooms`):
    *   Use `mist.router` or `wisp`.
    *   Implement token generation.
2.  **WebSockets** (`/api/rooms/:id/connect`):
    *   Use `mist.websocket`.
    *   **On Init**: Look up the Room Actor. Send `Join` message to it, passing the WebSocket's own `Subject`.
    *   **On Text**: Decode JSON `UserAction` -> Send to Room Actor.
    *   **On Custom Message (from Room)**: Encode `RoomUpdate` to JSON -> Send frame to client.
    *   **On Close**: Send `Leave` to Room Actor.

### Phase 4: Integration
1.  Dockerize: Use the `erlang:alpine` base image.
2.  Update `docker-compose.yml` to point to the Gleam build.

---

## 4. Key Differences & Challenges

1.  **JSON Boilerplate**: You lose `#[derive(Serialize)]`. You will need to write decoders (using `gleam/dynamic`) and encoders (using `gleam/json`) manually.
2.  **Global State**: Instead of `Lazy<FilterConfig>`, you might pass the config into the Supervisor start argument, or use a named persistent_term (Erlang optimization) for static read-only data like the word lists.
3.  **Mutable Data**: There is no mutation. Updating the `RoomState` inside the actor means returning a *new* record record with the changes.
    ```gleam
    // Rust
    room.messages.push(msg);
    // Gleam
    actor.continue(State(..state, messages: [msg, ..state.messages]))
    ```

## 5. Recommended Directory Structure

```text
src/
├── babel.gleam           # Application Entrypoint (Supervisor)
├── types.gleam           # Common types (RoomId, UserAction)
├── logic/
│   ├── filter.gleam      # Censorship logic
│   └── words.gleam       # Word list loading
├── actors/
│   ├── room.gleam        # The Room GenServer
│   └── manager.gleam     # Room Registry/Supervisor
└── web/
    ├── router.gleam      # HTTP handlers
    └── socket.gleam      # WebSocket handler (Mist adapter)
```
