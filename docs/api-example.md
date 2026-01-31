# Backend API Reference

**Base URL:** `http://localhost:3000`

## Authentication

All authenticated endpoints require the `X-User-Token` header.

**Test tokens:**
| Token | User ID | Country |
|-------|---------|---------|
| `test-token-alice` | alice | A |
| `test-token-bob` | bob | B |
| `test-token-charlie` | charlie | C |
| `test-token` | testuser | A |

---

## HTTP Endpoints

### GET /api/info

Returns server information and filter configuration.

**Response:**
```json
{
  "filter_config": {
    "A": ["freedom", "democracy", "protest"],
    "B": ["monarchy", "tradition", "heritage"],
    "C": ["capitalism", "profit", "market"]
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

---

## WebSocket Endpoints

### WS /api/rooms/:roomId/connect

Connect as a participant (can send messages). Requires authentication via `X-User-Token` header.

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

**send note:**
```json
{
  "send_note": {
    "A": ["wordA", "wordB"],
    "B": [],
  }
}
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
- **Country D:** capitalism, profit, market

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 403 | Invalid or missing X-User-Token |
| 404 | Room not found |
| 101 | WebSocket upgrade successful |

---