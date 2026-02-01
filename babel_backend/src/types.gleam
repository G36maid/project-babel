import gleam/dict
import gleam/option

// Type aliases
pub type RoomId =
  String

pub type UserId =
  String

pub type MessageId =
  Int

pub type CountryCode =
  String

pub type Timestamp =
  Int

// Constants
pub const censorship_replacement = "***"

// Core types
pub type Message {
  Message(
    id: Int,
    sender_id: String,
    sender_country: String,
    content: String,
    timestamp: Int,
  )
}

pub type CensoredMessage {
  CensoredMessage(id: Int, sender_id: String, content: String, was_censored: Bool)
}

pub type UserAction {
  SendMessage(content: String)
  SendMessageArray(words: List(String))
  SubmitNotes(notes: dict.Dict(String, List(String)))
  LeaveRoom
}

pub type Participant {
  Participant(user_id: String, country: String, joined_at: Int)
}

pub type RoomState {
  RoomState(
    room_id: String,
    participants: List(Participant),
    recent_messages: List(CensoredMessage),
  )
}

pub type Notification {
  Notification(message: String)
}

pub type PlayerProgress {
  PlayerProgress(
    user_id: String,
    country: String,
    discovered_count: Int,
    total_required: Int,
    completed: Bool,
  )
}

pub type VictoryState {
  VictoryState(
    achieved: Bool,
    player_progress: List(PlayerProgress),
    unlocked_at: option.Option(Int),
  )
}

pub type RoomUpdate {
  RoomUpdate(
    room_state: RoomState,
    new_messages: List(Message),
    notifications: List(Notification),
    room_closed: Bool,
    victory: option.Option(VictoryState),
  )
}

pub type FilterConfig {
  FilterConfig(banned_words: dict.Dict(String, List(String)))
}

pub type ClientRoomUpdate {
  ClientRoomUpdate(
    room_state: RoomState,
    new_messages: List(CensoredMessage),
    notifications: List(Notification),
    room_closed: Bool,
    victory: option.Option(VictoryState),
  )
}
