use std::time::{SystemTime, UNIX_EPOCH};

use crate::data::*;
use crate::filter::CensorshipFilter;
use crate::words::{load_words, generate_allowed_and_banned_words};

pub struct ChatRoom {
    room_id: RoomId,
    #[allow(dead_code)]
    config: &'static FilterConfig,
    participants: Vec<Participant>,
    messages: Vec<Message>,
    message_counter: MessageId,
    pub(crate) filter: CensorshipFilter,
    pub(crate) allowed_words: Vec<String>,
    pub sender_censor: bool,
    pub receiver_censor: bool,
    pub shadow_ban: bool,
}

impl ChatRoom {
    pub fn new(room_id: RoomId, config: &'static FilterConfig) -> Self {
        // Load words from words.json
        let words = load_words("words.json");
        let country_codes = ["TW", "CN", "US", "RU"];
        let (allowed_words, banned_map) = generate_allowed_and_banned_words(&words, &country_codes);
        // Clone and update the config's banned_words for this room
        let mut config_owned = config.clone();
        for (country, banned) in &banned_map {
            config_owned.banned_words.insert(country.clone(), banned.clone());
        }
        let config_ref: &'static FilterConfig = Box::leak(Box::new(config_owned));
        Self {
            room_id,
            config: config_ref,
            participants: Vec::new(),
            messages: Vec::new(),
            message_counter: 0,
            filter: CensorshipFilter::new(config_ref),
            allowed_words,
            sender_censor: true,
            receiver_censor: true,
            shadow_ban: true,
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
            // Accept only allowed words in message array
            UserAction::SendMessageArray(words) => {
                // Only keep allowed words
                let filtered: Vec<String> = words
                    .into_iter()
                    .filter(|w| self.allowed_words.contains(w))
                    .collect();
                let content = filtered.join(" ");
                if content.is_empty() {
                    return (None, notifications);
                }
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
            // fallback for old SendMessage (single string)
            UserAction::SendMessage(content) => {
                // fallback: split and filter
                let filtered: Vec<String> = content
                    .split_whitespace()
                    .filter(|w| self.allowed_words.contains(&w.to_string()))
                    .map(|w| w.to_string())
                    .collect();
                let content = filtered.join(" ");
                if content.is_empty() {
                    return (None, notifications);
                }
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
        }
    }

    /// Get censored room state for a specific country
    pub fn get_censored_state_for(&self, country: &CountryCode) -> RoomState {
        let censored_messages = self
            .messages
            .iter()
            .map(|msg| {
                let sender = if self.sender_censor { Some(&msg.sender_country) } else { None };
                let receiver = if self.receiver_censor { Some(country) } else { None };
                // shadow_ban: if sender==receiver, skip censorship
                let (content, was_censored) = if self.shadow_ban && sender.is_some() && receiver.is_some() && sender == receiver {
                    (msg.content.clone(), false)
                } else {
                    self.filter.censor_message(&msg.content, sender, receiver)
                };
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
        let sender = if self.sender_censor { Some(&message.sender_country) } else { None };
        let receiver = if self.receiver_censor { Some(country) } else { None };
        let (content, was_censored) = if self.shadow_ban && sender.is_some() && receiver.is_some() && sender == receiver {
            (message.content.clone(), false)
        } else {
            self.filter.censor_message(&message.content, sender, receiver)
        };
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
