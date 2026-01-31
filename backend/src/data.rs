use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::room::ChatRoom;

pub type RoomId = String;
pub type UserId = String;
pub type MessageId = u64;
pub type CountryCode = String;
pub type Timestamp = u64;

pub const MAX_USER_ACTIONS: usize = 100;

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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    SendMessage(String),
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
pub struct RoomUpdate {
    pub room_state: RoomState,
    pub new_messages: Vec<CensoredMessage>,
    pub notifications: Vec<Notification>,
    pub room_closed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub banned_words: HashMap<CountryCode, Vec<String>>,
    pub replacement: String,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            banned_words: HashMap::new(),
            replacement: "***".to_string(),
        }
    }
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
