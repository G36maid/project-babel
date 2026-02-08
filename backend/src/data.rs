use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

pub type RoomId = String;
pub type UserId = String;
pub type MessageId = u64;
pub type CountryCode = String;
pub type Timestamp = u64;

pub const MAX_USER_ACTIONS: usize = 100;

/// The replacement string used when censoring banned words.
pub const CENSORSHIP_REPLACEMENT: &str = "***";

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub id: MessageId,
    pub sender_id: UserId,
    pub sender_country: CountryCode,
    pub content: String,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CensoredMessage {
    pub id: MessageId,
    pub sender_id: UserId,
    pub content: String,
    pub was_censored: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserAction {
    SendMessage(String),
    SendMessageArray(Vec<String>),
    SubmitNotes(HashMap<CountryCode, Vec<String>>),
    LeaveRoom,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Participant {
    pub user_id: UserId,
    pub country: CountryCode,
    pub joined_at: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct RoomState {
    pub room_id: RoomId,
    pub participants: Vec<Participant>,
    pub recent_messages: Vec<CensoredMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Notification {
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct PlayerProgress {
    pub user_id: UserId,
    pub country: CountryCode,
    pub discovered_count: usize,
    pub total_required: usize,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct VictoryState {
    pub achieved: bool,
    pub player_progress: Vec<PlayerProgress>,
    pub unlocked_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct RoomUpdate {
    pub room_state: RoomState,
    /// Raw messages that need to be censored per-user before sending to clients.
    pub new_messages: Vec<Message>,
    pub notifications: Vec<Notification>,
    pub room_closed: bool,
    pub victory: Option<VictoryState>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, ToSchema)]
pub struct FilterConfig {
    pub banned_words: HashMap<CountryCode, Vec<String>>,
}

/// Trait defining the core behavior of a game room.
///
/// This trait abstracts room management, allowing different room implementations
/// while maintaining consistent server behavior.
pub trait Room: Send + Sync {
    /// Get the room's unique identifier.
    fn room_id(&self) -> &RoomId;

    /// Get the list of current participants.
    fn participants(&self) -> &[Participant];

    /// Check if the room has no participants.
    fn is_empty(&self) -> bool;

    /// Add a participant to the room.
    /// Returns true if the participant was added, false if already present.
    fn add_participant(&mut self, user_id: UserId, country: CountryCode) -> bool;

    /// Remove a participant from the room.
    /// Returns true if the participant was removed, false if not found.
    fn remove_participant(&mut self, user_id: &UserId) -> bool;

    /// Process a user action and return the resulting message and notifications.
    fn process_action(
        &mut self,
        user_id: &UserId,
        country: &CountryCode,
        action: UserAction,
    ) -> (Option<Message>, Vec<Notification>);

    /// Get the room state censored for a specific country.
    fn get_censored_state_for(&self, country: &CountryCode) -> RoomState;

    /// Censor a single message for a specific country.
    fn censor_message_for(&self, message: &Message, country: &CountryCode) -> CensoredMessage;

    /// Trigger victory condition (used when puzzle is solved).
    fn win(&mut self);

    /// Get all player notes.
    fn get_player_notes(&self) -> &HashMap<UserId, HashMap<CountryCode, Vec<String>>>;

    /// Get player progress for all participants.
    fn get_player_progress(&self) -> Vec<PlayerProgress>;

    /// Check if victory conditions are met.
    fn check_victory(&mut self) -> bool;

    /// Get the current victory state.
    fn get_victory_state(&self) -> VictoryState;

    /// Get the filter configuration.
    fn filter_config(&self) -> &FilterConfig;

    /// Get the list of allowed words.
    fn allowed_words(&self) -> &[String];
}

pub trait RoomConfig: Send + Sync {
    fn get_filter_config(&self) -> &'static FilterConfig;
    fn init_room(&self, room_id: RoomId) -> Box<dyn Room>;
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct RoomInfo {
    pub filter_config: FilterConfig,
}

pub struct UserMessage {
    pub user_id: UserId,
    pub country: CountryCode,
    pub action: UserAction,
}
