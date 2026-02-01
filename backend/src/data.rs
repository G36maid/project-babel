use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::room::ChatRoom;

pub type RoomId = String;
pub type UserId = String;
pub type MessageId = u64;
pub type CountryCode = String;
pub type Timestamp = u64;

pub const MAX_USER_ACTIONS: usize = 100;

/// The replacement string used when censoring banned words.
pub const CENSORSHIP_REPLACEMENT: &str = "***";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub sender_id: UserId,
    pub sender_country: CountryCode,
    pub content: String,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CensoredMessage {
    pub id: MessageId,
    pub sender_id: UserId,
    pub content: String,
    pub was_censored: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    SendMessage(String),
    SendMessageArray(Vec<String>),
    SubmitNotes(HashMap<CountryCode, Vec<String>>),
    LeaveRoom,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: UserId,
    pub country: CountryCode,
    pub joined_at: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: RoomId,
    pub participants: Vec<Participant>,
    pub recent_messages: Vec<CensoredMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerProgress {
    pub user_id: UserId,
    pub country: CountryCode,
    pub discovered_count: usize,
    pub total_required: usize,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VictoryState {
    pub achieved: bool,
    pub player_progress: Vec<PlayerProgress>,
    pub unlocked_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomUpdate {
    pub room_state: RoomState,
    /// Raw messages that need to be censored per-user before sending to clients.
    pub new_messages: Vec<Message>,
    pub notifications: Vec<Notification>,
    pub room_closed: bool,
    pub victory: Option<VictoryState>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FilterConfig {
    pub banned_words: HashMap<CountryCode, Vec<String>>,
}

pub trait RoomConfig: Send + Sync {
    fn get_filter_config(&self) -> &'static FilterConfig;
    fn init_room(&self, room_id: RoomId) -> ChatRoom;
}

#[derive(Clone, Debug, Serialize)]
pub struct RoomInfo {
    pub filter_config: FilterConfig,
}

pub struct UserMessage {
    pub user_id: UserId,
    pub country: CountryCode,
    pub action: UserAction,
}
