# Backend API Reference

**Base URL:** `http://localhost:3000`

## Authentication

Most endpoints require a token obtained via the login endpoint.
For HTTP requests, pass the token in the `X-User-Token` header.
For WebSocket connections, pass the token as a query parameter `?token=<token>`.

---

## HTTP Endpoints

### POST /api/login

Creates a new session and returns an authentication token.

**Request:**
```json
{
  "username": "alice",
  "country": "A"
}
```

**Response:**
```json
{
  "token": "a1b2c3d4e5f6g7h8"
}
```

### GET /api/info

Returns server information and global filter configuration.

**Response:**
```json
{
  "filter_config": {
    "banned_words": {
      "A": ["freedom", "democracy", "protest"],
      "B": ["monarchy", "tradition", "heritage"],
      "C": ["capitalism", "profit", "market"]
    }
  }
}
```

### GET /api/rooms

Lists all active room IDs.

**Response:**
```json
["AbCdEf1234567890", "XyZ9876543210abc"]
```

### POST /api/rooms

Creates a new room. Requires authentication.

**Headers:**
- `X-User-Token: <token>`

**Response:**
```json
"AbCdEf1234567890"
```

**Errors:**
- `403 Forbidden` - Invalid or missing token

### GET /api/rooms/:roomId/info

Returns allowed and banned words for the specific room.

**Response:**
```json
{
  "allowed_words": ["apple", "banana"],
  "banned_words": {
    "A": ["badword"]
  }
}
```

### POST /api/rooms/:roomId/solve

Submits a solution for the censorship puzzle. Requires authentication.

**Headers:**
- `X-User-Token: <token>`

**Request:**
```json
{
  "answer": {
    "A": ["word1", "word2"],
    "B": ["word3"]
  }
}
```

**Response:**
```json
{
  "solved": true
}
```

---

## WebSocket Endpoints

### WS /api/rooms/:roomId/connect?token=<token>

Connect as a participant (can send messages).
**Query Parameters:**
- `token`: The authentication token obtained from `/api/login`.

### WS /api/rooms/:roomId/spectate

Connect as a spectator (read-only). No authentication required.

---

## Message Formats

### Client to Server (Participant Only)

**Send Message:**
```json
{"send_message": "Your message content here"}
```

**Send Message (Array):**
```json
{"send_message_array": ["word1", "word2"]}
```

**Send Note:**
```json
{
  "send_note": {
    "A": ["wordA", "wordB"],
    "B": []
  }
}
```

**Leave Room:**
```json
"leave_room"
```

### Server → Client (RoomUpdate)

```json
{
  "room_state": {
    "room_id": "AbCdEf1234567890",
    "participants": [
      {
        "user_id": "alice",
        "country": "A",
        "joined_at": 1706750400
      }
    ],
    "recent_messages": [
      {
        "id": 1,
        "sender_id": "alice",
        "content": "Hello everyone!",
        "was_censored": false
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
    {"message": "bob joined the room"}
  ],
  "room_closed": false
}
```

---

## Censorship Rules

Messages are filtered based on both sender's and receiver's country rules. Banned words are replaced with `***` (case-insensitive).

**Default banned words:**
- **Country A:** freedom, democracy, protest
- **Country B:** monarchy, tradition, heritage
- **Country C:** capitalism, profit, market

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 403 | Invalid or missing token |
| 404 | Room not found |
| 101 | WebSocket upgrade successful |
