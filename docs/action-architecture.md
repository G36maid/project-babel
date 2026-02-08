# Action Architecture Refactoring

## Overview

This document describes the refactoring of the action system in Project Babel's backend, which separates system-level room management actions from game-specific actions.

## Motivation

Previously, the `UserAction` enum mixed generic chat/room actions with game-specific logic:

```rust
// Before
enum UserAction {
    SendMessage(String),          // System action
    SendMessageArray(Vec<String>), // System action
    SubmitNotes(HashMap<...>),     // Game action
    LeaveRoom,                      // System action
}
```

**Problems:**
- Unclear separation between infrastructure and game logic
- Difficult to extend with new game mechanics
- Room management coupled with game rules
- Hard to test independently

## Solution

We introduced a layered action architecture:

```rust
// System-level actions (handled by Room/RoomManager)
enum SystemAction {
    SendMessage(String),
    SendMessageArray(Vec<String>),
    LeaveRoom,
}

// Game-specific actions (delegated to GameRules)
enum GameAction {
    SubmitNotes(HashMap<CountryCode, Vec<String>>),
}

// Transport layer envelope
enum UserAction {
    System(SystemAction),
    Game(GameAction),
    // Legacy variants for backward compatibility
    SendMessage(String),
    SendMessageArray(Vec<String>),
    SubmitNotes(HashMap<...>),
    LeaveRoom,
}
```

## Processing Flow

```
┌─────────────────────────────────────────────────────┐
│              Client (WebSocket/HTTP)                 │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
           ┌───────────────────────┐
           │     UserAction        │
           │   (Transport Layer)   │
           └───────────┬───────────┘
                       │
         ┌─────────────┴─────────────┐
         ▼                           ▼
┌────────────────┐         ┌─────────────────┐
│ SystemAction   │         │  GameAction     │
│                │         │                 │
│ - SendMessage  │         │ - SubmitNotes   │
│ - SendMsg[]    │         │                 │
│ - LeaveRoom    │         │                 │
└────────┬───────┘         └────────┬────────┘
         │                          │
         ▼                          ▼
┌─────────────────────┐   ┌──────────────────┐
│process_system_action│   │process_game_action│
│                     │   │                   │
│ • Create messages   │   │ • Store notes     │
│ • Manage users      │   │ • Game logic      │
│ • Room state        │   │ • Victory checks  │
└─────────────────────┘   └──────────────────┘
```

## Implementation Details

### ChatRoom Action Processing

```rust
impl ChatRoom {
    // Main entry point
    pub fn process_action(
        &mut self,
        user_id: &UserId,
        country: &CountryCode,
        action: UserAction,
    ) -> (Option<Message>, Vec<Notification>) {
        match action {
            // New structured actions
            UserAction::System(sys_action) => 
                self.process_system_action(user_id, country, sys_action),
            UserAction::Game(game_action) => 
                self.process_game_action(user_id, game_action),
            
            // Legacy actions (backward compatibility)
            UserAction::SendMessage(content) => 
                self.process_system_action(user_id, country, 
                    SystemAction::SendMessage(content)),
            // ... other legacy variants
        }
    }

    // System action handler (room management)
    fn process_system_action(
        &mut self,
        user_id: &UserId,
        country: &CountryCode,
        action: SystemAction,
    ) -> (Option<Message>, Vec<Notification>) {
        // Handles: message creation, user leave, etc.
        // Uses: self.messages, self.participants, self.game.is_word_allowed()
    }

    // Game action handler (delegates to GameRules)
    fn process_game_action(
        &mut self,
        user_id: &UserId,
        action: GameAction,
    ) -> (Option<Message>, Vec<Notification>) {
        // Handles: player notes, game-specific logic
        // Delegates to: self.game.submit_player_notes()
    }
}
```

## Benefits

### 1. Separation of Concerns
- **System logic** in `process_system_action` - message handling, room management
- **Game logic** in `process_game_action` - player notes, victory conditions
- Clear boundaries between infrastructure and game mechanics

### 2. Extensibility
Adding new game mechanics is now easier:

```rust
// Adding a new game action
enum GameAction {
    SubmitNotes(HashMap<...>),
    SubmitSolution(Solution),  // New!
    RequestHint,                // New!
}

// Only need to update process_game_action
fn process_game_action(...) {
    match action {
        GameAction::SubmitNotes(...) => { /* existing */ }
        GameAction::SubmitSolution(solution) => { /* new logic */ }
        GameAction::RequestHint => { /* new logic */ }
    }
}
```

### 3. Better Testing
Can now test system and game logic independently:

```rust
#[test]
fn test_system_action_send_message() {
    let action = UserAction::System(
        SystemAction::SendMessage("hello".to_string())
    );
    // Test only room management logic
}

#[test]
fn test_game_action_submit_notes() {
    let action = UserAction::Game(
        GameAction::SubmitNotes(notes)
    );
    // Test only game logic
}
```

### 4. Type Safety
The compiler enforces proper routing:

```rust
// This won't compile - type mismatch
process_system_action(user_id, country, GameAction::SubmitNotes(...));

// Correct - types match
process_game_action(user_id, GameAction::SubmitNotes(...));
```

### 5. Backward Compatibility
Existing clients continue to work:

```json
// Old format still works
{"leave_room": null}
{"send_message": "hello"}
{"submit_notes": {"A": ["freedom"]}}

// New format also works
{"system": {"leave_room": null}}
{"system": {"send_message": "hello"}}
{"game": {"submit_notes": {"A": ["freedom"]}}}
```

## Migration Path

### For New Code
Use structured actions:

```rust
// Prefer this
UserAction::System(SystemAction::SendMessage("hello".to_string()))
UserAction::Game(GameAction::SubmitNotes(notes))

// Over this (legacy)
UserAction::SendMessage("hello".to_string())
UserAction::SubmitNotes(notes)
```

### For Existing Code
No changes required - legacy variants are maintained for backward compatibility.

### Future Deprecation (Optional)
Once all clients migrate:

1. Mark legacy variants as deprecated
2. Add deprecation warnings
3. Eventually remove after grace period

```rust
#[deprecated(note = "Use UserAction::System(SystemAction::SendMessage) instead")]
SendMessage(String),
```

## Testing

### Test Coverage

**New tests added:**
- `test_system_action_send_message` - Tests new SystemAction::SendMessage
- `test_system_action_send_message_array` - Tests new SystemAction::SendMessageArray
- `test_system_action_leave_room` - Tests new SystemAction::LeaveRoom
- `test_game_action_submit_notes` - Tests new GameAction::SubmitNotes
- `test_legacy_actions_still_work` - Ensures backward compatibility

**Test results:**
```
20 tests passed, 0 failed
- 15 existing tests: ✅ all passing
- 5 new tests: ✅ all passing
```

## API Changes

### OpenAPI Schema Updated

Added new types to OpenAPI documentation:
- `SystemAction` - enum with system-level actions
- `GameAction` - enum with game-specific actions
- `UserAction` - updated to include new variants

This ensures API documentation stays current and clients can discover new action types.

## Performance Impact

**No performance degradation:**
- Same number of allocations
- Simple enum matching (compiler optimizes well)
- No additional heap allocations
- Pattern matching is compile-time optimized

## Future Enhancements

This architecture enables several future improvements:

1. **Multiple Game Modes**: Easy to add different game rulesets
   ```rust
   enum GameAction {
       Censorship(CensorshipAction),
       Puzzle(PuzzleAction),
       Quiz(QuizAction),
   }
   ```

2. **Action Middleware**: Can add logging, metrics, rate limiting per action type
   ```rust
   fn process_system_action(...) {
       metrics::increment("system_action", action.name());
       // ... existing logic
   }
   ```

3. **Action Validation**: Type-safe validation per action category
   ```rust
   trait ValidateAction {
       fn validate(&self) -> Result<(), ValidationError>;
   }
   ```

## Conclusion

The action architecture refactoring successfully separates concerns between system-level room management and game-specific logic while maintaining full backward compatibility. This provides a solid foundation for future game mechanics and improvements.

**Key achievements:**
- ✅ Clear separation of concerns
- ✅ Improved extensibility
- ✅ Better testability
- ✅ Type safety
- ✅ Backward compatibility
- ✅ No performance degradation
- ✅ Updated documentation
