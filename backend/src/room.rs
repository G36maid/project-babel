use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::*;
use crate::filter::CensorshipFilter;

pub struct ChatRoom {
    room_id: RoomId,
    #[allow(dead_code)]
    config: &'static FilterConfig,
    participants: Vec<Participant>,
    messages: Vec<Message>,
    message_counter: MessageId,
    filter: CensorshipFilter,
}

impl ChatRoom {
    pub fn new(room_id: RoomId, config: &'static FilterConfig) -> Self {
        Self {
            room_id,
            config,
            participants: Vec::new(),
            messages: Vec::new(),
            message_counter: 0,
            filter: CensorshipFilter::new(config),
        }
    }

    fn current_timestamp() -> Timestamp {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn add_participant(&mut self, user_id: UserId, country: CountryCode) -> bool {
        if self.participants.iter().any(|p| p.user_id == user_id) {
            return false;
        }

        self.participants.push(Participant {
            user_id,
            country,
            joined_at: Self::current_timestamp(),
        });
        true
    }

    pub fn remove_participant(&mut self, user_id: &UserId) -> bool {
        let initial_len = self.participants.len();
        self.participants.retain(|p| &p.user_id != user_id);
        self.participants.len() < initial_len
    }

    /// Pure function - processes action and returns results without I/O
    pub fn process_action(
        &mut self,
        user_id: &UserId,
        country: &CountryCode,
        action: UserAction,
    ) -> (Option<Message>, Vec<Notification>) {
        let mut notifications = Vec::new();

        match action {
            UserAction::SendMessage(content) => {
                self.message_counter += 1;
                let message = Message {
                    id: self.message_counter,
                    sender_id: user_id.clone(),
                    sender_country: country.clone(),
                    content,
                    timestamp: Self::current_timestamp(),
                };
                self.messages.push(message.clone());
                (Some(message), notifications)
            }
            UserAction::LeaveRoom => {
                if self.remove_participant(user_id) {
                    notifications.push(Notification {
                        message: format!("{} left the room", user_id),
                    });
                }
                (None, notifications)
            }
        }
    }

    /// Get censored room state for a specific country
    pub fn get_censored_state_for(&self, country: &CountryCode) -> RoomState {
        let censored_messages = self
            .messages
            .iter()
            .map(|msg| {
                let (content, was_censored) =
                    self.filter.censor_message(&msg.content, &msg.sender_country, country);
                CensoredMessage {
                    id: msg.id,
                    sender_id: msg.sender_id.clone(),
                    content,
                    was_censored,
                }
            })
            .collect();

        RoomState {
            room_id: self.room_id.clone(),
            participants: self.participants.clone(),
            recent_messages: censored_messages,
        }
    }

    /// Censor a single message for a specific country
    pub fn censor_message_for(&self, message: &Message, country: &CountryCode) -> CensoredMessage {
        let (content, was_censored) =
            self.filter.censor_message(&message.content, &message.sender_country, country);
        CensoredMessage {
            id: message.id,
            sender_id: message.sender_id.clone(),
            content,
            was_censored,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.participants.is_empty()
    }

    pub fn room_id(&self) -> &RoomId {
        &self.room_id
    }

    pub fn participants(&self) -> &[Participant] {
        &self.participants
    }
}
