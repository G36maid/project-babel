# Backend API Examples

This document provides examples for interacting with the Project Babel backend API.

**Base URL:** `http://localhost:3000`

## Authentication

The API uses token-based authentication via the `X-User-Token` header.

**Available test tokens:**
| Token | User ID | Country |
|-------|---------|---------|
| `test-token-alice` | alice | A |
| `test-token-bob` | bob | B |
| `test-token-charlie` | charlie | C |
| `test-token` | testuser | A |

---

## HTTP API Examples

### 1. Get Server Info (Filter Configuration)

```bash
curl http://localhost:3000/api/info
```

**Response:**
```json
{
  "filter_config": {
    "banned_words": {
      "A": ["freedom", "democracy", "protest"],
      "B": ["monarchy", "tradition", "heritage"],
      "C": ["capitalism", "profit", "market"]
    },
    "replacement": "***"
  }
}
```

### 2. List All Rooms

```bash
curl http://localhost:3000/api/rooms
```

**Response:**
```json
["AbCdEf1234567890", "XyZ9876543210abc"]
```

### 3. Create a New Room (Requires Authentication)

```bash
curl -X POST http://localhost:3000/api/rooms \
  -H "X-User-Token: test-token-alice"
```

**Response:**
```json
"AbCdEf1234567890"
```

**Error (invalid token):**
```
403 Forbidden
```

---

## WebSocket API Examples

### JavaScript/TypeScript Examples

#### Participant Connection (Can Send Messages)

```javascript
// Connect as a participant (requires authentication)
const roomId = "AbCdEf1234567890";
const token = "test-token-alice";

const ws = new WebSocket(`ws://localhost:3000/api/rooms/${roomId}/connect`, [], {
  headers: {
    "X-User-Token": token
  }
});

ws.onopen = () => {
  console.log("Connected to room as participant");
};

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log("Room update:", update);

  // Handle room state
  if (update.room_state) {
    console.log("Participants:", update.room_state.participants);
    console.log("Recent messages:", update.room_state.recent_messages);
  }

  // Handle new messages
  if (update.new_messages && update.new_messages.length > 0) {
    update.new_messages.forEach(msg => {
      console.log(`[${msg.sender_id}]: ${msg.content}`);
      if (msg.was_censored) {
        console.log("  (This message was censored)");
      }
    });
  }

  // Handle notifications (join/leave)
  if (update.notifications) {
    update.notifications.forEach(n => console.log("Notification:", n.message));
  }

  // Handle room closure
  if (update.room_closed) {
    console.log("Room has been closed");
    ws.close();
  }
};

ws.onerror = (error) => {
  console.error("WebSocket error:", error);
};

ws.onclose = () => {
  console.log("Disconnected from room");
};

// Send a message
function sendMessage(content) {
  ws.send(JSON.stringify({ send_message: content }));
}

// Leave the room
function leaveRoom() {
  ws.send(JSON.stringify("leave_room"));
}

// Example usage:
// sendMessage("Hello everyone!");
// leaveRoom();
```

#### Spectator Connection (Read-Only)

```javascript
// Connect as a spectator (no authentication required)
const roomId = "AbCdEf1234567890";

const ws = new WebSocket(`ws://localhost:3000/api/rooms/${roomId}/spectate`);

ws.onopen = () => {
  console.log("Connected as spectator");
};

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log("Room update:", update);

  // Spectators receive the same RoomUpdate as participants
  // but cannot send messages
};

ws.onclose = () => {
  console.log("Spectator connection closed");
};
```

### Python Examples

#### Using `websockets` Library

```python
import asyncio
import json
import websockets

async def participant_example():
    """Connect as a participant and send messages."""
    room_id = "AbCdEf1234567890"
    token = "test-token-alice"
    uri = f"ws://localhost:3000/api/rooms/{room_id}/connect"

    extra_headers = {"X-User-Token": token}

    async with websockets.connect(uri, extra_headers=extra_headers) as ws:
        print("Connected as participant")

        # Send a message
        await ws.send(json.dumps({"send_message": "Hello from Python!"}))

        # Receive updates
        async for message in ws:
            update = json.loads(message)
            print(f"Received update: {json.dumps(update, indent=2)}")

            if update.get("room_closed"):
                print("Room closed")
                break

            # Process new messages
            for msg in update.get("new_messages", []):
                sender = msg["sender_id"]
                content = msg["content"]
                censored = msg["was_censored"]
                print(f"[{sender}]: {content}" + (" (censored)" if censored else ""))

async def spectator_example():
    """Connect as a spectator (read-only)."""
    room_id = "AbCdEf1234567890"
    uri = f"ws://localhost:3000/api/rooms/{room_id}/spectate"

    async with websockets.connect(uri) as ws:
        print("Connected as spectator")

        async for message in ws:
            update = json.loads(message)
            print(f"Spectator received: {json.dumps(update, indent=2)}")

            if update.get("room_closed"):
                break

# Run:
# asyncio.run(participant_example())
# asyncio.run(spectator_example())
```

#### Using `requests` for HTTP + `websocket-client` for WebSocket

```python
import requests
import json
from websocket import create_connection

BASE_URL = "http://localhost:3000"
TOKEN = "test-token-alice"

# HTTP: Get server info
def get_info():
    response = requests.get(f"{BASE_URL}/api/info")
    return response.json()

# HTTP: List rooms
def list_rooms():
    response = requests.get(f"{BASE_URL}/api/rooms")
    return response.json()

# HTTP: Create room
def create_room():
    response = requests.post(
        f"{BASE_URL}/api/rooms",
        headers={"X-User-Token": TOKEN}
    )
    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"Failed to create room: {response.status_code}")

# WebSocket: Connect and chat
def chat_in_room(room_id):
    ws = create_connection(
        f"ws://localhost:3000/api/rooms/{room_id}/connect",
        header={"X-User-Token": TOKEN}
    )

    # Send a message
    ws.send(json.dumps({"send_message": "Hello!"}))

    # Receive updates
    while True:
        result = ws.recv()
        update = json.loads(result)
        print(f"Update: {update}")

        if update.get("room_closed"):
            break

    ws.close()

# Example workflow:
# print(get_info())
# rooms = list_rooms()
# room_id = create_room()
# chat_in_room(room_id)
```

### cURL + websocat Examples

#### HTTP Requests with cURL

```bash
# Get server info
curl http://localhost:3000/api/info | jq

# List rooms
curl http://localhost:3000/api/rooms | jq

# Create a room
curl -X POST http://localhost:3000/api/rooms \
  -H "X-User-Token: test-token-alice" | jq
```

#### WebSocket with websocat

```bash
# Install websocat: cargo install websocat

# Connect as participant
websocat -H "X-User-Token: test-token-alice" \
  ws://localhost:3000/api/rooms/YOUR_ROOM_ID/connect

# Then type messages in JSON format:
# {"send_message": "Hello!"}
# "leave_room"

# Connect as spectator (no auth needed)
websocat ws://localhost:3000/api/rooms/YOUR_ROOM_ID/spectate
```

---

## Message Formats

### Client to Server (Participant Only)

**Send Message:**
```json
{"send_message": "Your message content here"}
```

**Leave Room:**
```json
"leave_room"
```

### Server to Client (RoomUpdate)

```json
{
  "room_state": {
    "room_id": "AbCdEf1234567890",
    "participants": [
      {
        "user_id": "alice",
        "country": "A",
        "joined_at": 1706750400
      },
      {
        "user_id": "bob",
        "country": "B",
        "joined_at": 1706750410
      }
    ],
    "recent_messages": [
      {
        "id": 1,
        "sender_id": "alice",
        "content": "Hello everyone!",
        "was_censored": false
      },
      {
        "id": 2,
        "sender_id": "bob",
        "content": "Let's discuss ***",
        "was_censored": true
      }
    ]
  },
  "new_messages": [
    {
      "id": 2,
      "sender_id": "bob",
      "content": "Let's discuss ***",
      "was_censored": true
    }
  ],
  "notifications": [
    {
      "message": "bob joined the room"
    }
  ],
  "room_closed": false
}
```

---

## Censorship System

Messages are filtered based on **both sender's and receiver's country rules**.

**Banned words by country:**
- **Country A:** freedom, democracy, protest
- **Country B:** monarchy, tradition, heritage
- **Country C:** capitalism, profit, market

**Example:**
- Alice (Country A) sends: "Fight for freedom and monarchy"
- Bob (Country B) receives: "Fight for *** and ***"
  - "freedom" censored (banned in sender's country A)
  - "monarchy" censored (banned in receiver's country B)

---

## Complete Workflow Example (JavaScript)

```javascript
const BASE_URL = "http://localhost:3000";
const WS_URL = "ws://localhost:3000";

async function fullExample() {
  const token = "test-token-alice";

  // 1. Get server info
  const infoResponse = await fetch(`${BASE_URL}/api/info`);
  const info = await infoResponse.json();
  console.log("Filter config:", info.filter_config);

  // 2. Create a room
  const createResponse = await fetch(`${BASE_URL}/api/rooms`, {
    method: "POST",
    headers: { "X-User-Token": token }
  });
  const roomId = await createResponse.json();
  console.log("Created room:", roomId);

  // 3. Connect to room via WebSocket
  const ws = new WebSocket(`${WS_URL}/api/rooms/${roomId}/connect`, [], {
    headers: { "X-User-Token": token }
  });

  ws.onopen = () => {
    console.log("Connected!");

    // Send a test message
    ws.send(JSON.stringify({ send_message: "Hello, world!" }));
  };

  ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    console.log("Update:", update);
  };

  // Keep connection open for demo
  await new Promise(resolve => setTimeout(resolve, 5000));

  // 4. Leave room
  ws.send(JSON.stringify("leave_room"));
  ws.close();
}

fullExample().catch(console.error);
```

---

## Error Handling

| Status Code | Meaning |
|-------------|---------|
| 200 | Success |
| 403 | Forbidden - Invalid or missing X-User-Token |
| 404 | Room not found |
| 101 | WebSocket upgrade successful |

---

## Notes

1. **WebSocket Headers:** Browser WebSocket API doesn't support custom headers. For browser clients, consider implementing token passing via query parameters or initial message.

2. **Room Lifecycle:** Rooms automatically close when all participants leave.

3. **Message Limit:** Up to 100 actions are processed per update cycle.

4. **Censorship:** Content is filtered case-insensitively. Banned words are replaced with `***`.
