use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{debug, trace};

use crate::data::*;
use crate::filter::CensorshipFilter;
use crate::words::{generate_allowed_and_banned_words, load_words};

/// Game instructions message displayed when a room is created
const GAME_INSTRUCTIONS: &str = "Welcome to Project Babel! You are trying to communicate across a censorship firewall. Each country has different words that are banned. Work together to discover which words are censored for each country using the allowed symbols. Good luck!";

/// A chat room that manages participants, messages, and country-based censorship.
///
/// `ChatRoom` is the core game entity that handles real-time communication between
/// players from different countries. Messages are filtered based on country-specific
/// banned word lists, creating an asymmetric information environment where players
/// must deduce what words are censored for other countries.
///
/// # Censorship Mechanics
///
/// The room supports three censorship modes controlled by boolean flags:
/// - `sender_censor`: Censors words banned in the sender's country
/// - `receiver_censor`: Censors words banned in the receiver's country
/// - `shadow_ban`: When enabled, users see their own messages uncensored
///
/// # Example
///
/// ```ignore
/// let room = ChatRoom::new("room_1".to_string(), &config);
/// room.add_participant("alice".to_string(), "US".to_string());
/// let (msg, notifs) = room.process_action(&"alice", &"US", UserAction::SendMessage("hello".into()));
/// ```
pub struct ChatRoom {
    /// Unique identifier for this room.
    room_id: RoomId,
    /// Static reference to the filter configuration (banned words, replacement text).
    #[allow(dead_code)]
    config: &'static FilterConfig,
    /// List of participants currently in the room.
    participants: Vec<Participant>,
    /// All messages sent in this room.
    pub(crate) messages: Vec<Message>,
    /// Counter for generating unique message IDs.
    pub(crate) message_counter: MessageId,
    /// The censorship filter that processes messages based on country rules.
    pub(crate) filter: CensorshipFilter,
    /// Words that participants are allowed to use in messages.
    pub(crate) allowed_words: Vec<String>,
    /// Whether to apply censorship based on the sender's country.
    pub sender_censor: bool,
    /// Whether to apply censorship based on the receiver's country.
    pub receiver_censor: bool,
    /// Whether users see their own messages uncensored (shadow ban mode).
    pub shadow_ban: bool,
    /// Countries that are exempt from censorship.
    pub allowed: HashSet<String>,
    /// Player notes storing hypotheses about banned words per country.
    pub(crate) player_notes: HashMap<UserId, HashMap<CountryCode, Vec<String>>>,
}

impl ChatRoom {
    pub fn new(room_id: RoomId, config: &'static FilterConfig) -> Self {
        // Load words from words.json
        let words = load_words("words.json");
        let country_codes = ["A", "B", "C", "D"];
        let (allowed_words, banned_map) = generate_allowed_and_banned_words(&words, &country_codes);
        // Clone and update the config's banned_words for this room
        let mut config_owned = config.clone();
        for (country, banned) in &banned_map {
            config_owned
                .banned_words
                .insert(country.clone(), banned.clone());
        }
        let config_ref: &'static FilterConfig = Box::leak(Box::new(config_owned));
        
        // Create initial game instructions message
        let game_instructions = Message {
            id: 1,
            sender_id: "SYSTEM".to_string(),
            sender_country: "".to_string(),
            content: GAME_INSTRUCTIONS.to_string(),
            timestamp: Self::current_timestamp(),
        };
        
        Self {
            room_id,
            config: config_ref,
            participants: Vec::new(),
            messages: vec![game_instructions],
            message_counter: 1,
            filter: CensorshipFilter::new(config_ref),
            allowed_words,
            sender_censor: false,
            receiver_censor: true,
            shadow_ban: true,
            allowed: HashSet::new(),
            player_notes: HashMap::new(),
        }
    }

    pub fn current_timestamp() -> Timestamp {
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
            UserAction::SendNote(note_map) => {
                // Send note: players share their hypotheses about banned words
                // Store the latest notes for this user
                self.player_notes.insert(user_id.clone(), note_map.clone());

                // This generates a notification for other participants
                let country_count = note_map.len();
                let total_words: usize = note_map.values().map(|v| v.len()).sum();
                let country_label = if country_count == 1 {
                    "country"
                } else {
                    "countries"
                };
                let word_label = if total_words == 1 { "word" } else { "words" };
                notifications.push(Notification {
                    message: format!(
                        "{} shared exploration notes ({} {}, {} {})",
                        user_id, country_count, country_label, total_words, word_label
                    ),
                });
                (None, notifications)
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
        debug!(
            room_id = %self.room_id,
            viewer_country = %country,
            message_count = self.messages.len(),
            sender_censor = self.sender_censor,
            receiver_censor = self.receiver_censor,
            shadow_ban = self.shadow_ban,
            "Building censored room state"
        );

        let censored_messages: Vec<CensoredMessage> = self
            .messages
            .iter()
            .map(|msg| self.censor_message_for(msg, country))
            .collect();

        let censored_count = censored_messages.iter().filter(|m| m.was_censored).count();
        debug!(
            room_id = %self.room_id,
            viewer_country = %country,
            total_messages = censored_messages.len(),
            censored_count,
            "Room state censorship complete"
        );

        RoomState {
            room_id: self.room_id.clone(),
            participants: self.participants.clone(),
            recent_messages: censored_messages,
        }
    }

    /// Censor a single message for a specific country
    pub fn censor_message_for(&self, message: &Message, country: &CountryCode) -> CensoredMessage {
        trace!(
            message_id = message.id,
            sender_id = %message.sender_id,
            sender_country = %message.sender_country,
            viewer_country = %country,
            original_content = %message.content,
            "Processing message censorship"
        );

        // System messages are never censored
        if message.sender_id == "SYSTEM" {
            return CensoredMessage {
                id: message.id,
                sender_id: message.sender_id.clone(),
                content: message.content.clone(),
                was_censored: false,
            };
        }

        let sender = if self.sender_censor && !self.allowed.contains(&message.sender_country) {
            trace!(
                sender_country = %message.sender_country,
                "Applying sender censorship rules"
            );
            Some(&message.sender_country)
        } else {
            trace!(
                sender_censor = self.sender_censor,
                sender_in_allowed = self.allowed.contains(&message.sender_country),
                "Skipping sender censorship"
            );
            None
        };

        let receiver = if self.receiver_censor && !self.allowed.contains(country) {
            trace!(
                viewer_country = %country,
                "Applying receiver censorship rules"
            );
            Some(country)
        } else {
            trace!(
                receiver_censor = self.receiver_censor,
                viewer_in_allowed = self.allowed.contains(country),
                "Skipping receiver censorship"
            );
            None
        };

        let (content, was_censored) = if self.shadow_ban && &message.sender_country == country {
            debug!(
                message_id = message.id,
                sender_country = %message.sender_country,
                viewer_country = %country,
                "Shadow ban active: showing uncensored message to sender's country"
            );
            (message.content.clone(), false)
        } else {
            self.filter
                .censor_message(&message.content, sender, receiver)
        };

        debug!(
            message_id = message.id,
            viewer_country = %country,
            was_censored,
            original = %message.content,
            result = %content,
            "Censorship decision complete"
        );

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

    pub fn get_player_notes(&self) -> &HashMap<UserId, HashMap<CountryCode, Vec<String>>> {
        &self.player_notes
    }

    pub fn get_player_note(&self, user_id: &UserId) -> Option<&HashMap<CountryCode, Vec<String>>> {
        self.player_notes.get(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_config() -> &'static FilterConfig {
        let mut banned_words = HashMap::new();
        banned_words.insert("A".to_string(), vec!["freedom".to_string()]);
        banned_words.insert("B".to_string(), vec!["monarchy".to_string()]);
        Box::leak(Box::new(FilterConfig { banned_words }))
    }

    #[test]
    fn test_send_note_generates_notification() {
        let config = make_test_config();
        let mut room = ChatRoom::new("test_room".to_string(), config);

        let user_id = "alice".to_string();
        let country = "A".to_string();
        room.add_participant(user_id.clone(), country.clone());

        // Create a note with suspected banned words
        let mut note_map = HashMap::new();
        note_map.insert(
            "A".to_string(),
            vec!["freedom".to_string(), "democracy".to_string()],
        );
        note_map.insert("B".to_string(), vec!["monarchy".to_string()]);

        let action = UserAction::SendNote(note_map.clone());
        let (message, notifications) = room.process_action(&user_id, &country, action);

        // Should not create a message
        assert!(message.is_none());

        // Should generate a notification
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].message.contains("alice"));
        assert!(notifications[0].message.contains("exploration notes"));
        assert!(notifications[0].message.contains("2 countries"));
        assert!(notifications[0].message.contains("3 words"));

        // Should store the notes
        let stored_note = room.get_player_note(&user_id);
        assert!(stored_note.is_some());
        assert_eq!(stored_note.unwrap(), &note_map);
    }

    #[test]
    fn test_send_note_empty_map() {
        let config = make_test_config();
        let mut room = ChatRoom::new("test_room".to_string(), config);

        let user_id = "bob".to_string();
        let country = "B".to_string();
        room.add_participant(user_id.clone(), country.clone());

        // Send an empty note
        let note_map = HashMap::new();
        let action = UserAction::SendNote(note_map);
        let (message, notifications) = room.process_action(&user_id, &country, action);

        // Should not create a message
        assert!(message.is_none());

        // Should still generate a notification with 0 countries and 0 words
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].message.contains("bob"));
        assert!(notifications[0].message.contains("0 countries"));
        assert!(notifications[0].message.contains("0 words"));
    }

    #[test]
    fn test_send_note_single_country() {
        let config = make_test_config();
        let mut room = ChatRoom::new("test_room".to_string(), config);

        let user_id = "charlie".to_string();
        let country = "C".to_string();
        room.add_participant(user_id.clone(), country.clone());

        // Send note for single country
        let mut note_map = HashMap::new();
        note_map.insert(
            "D".to_string(),
            vec![
                "word1".to_string(),
                "word2".to_string(),
                "word3".to_string(),
            ],
        );

        let action = UserAction::SendNote(note_map);
        let (message, notifications) = room.process_action(&user_id, &country, action);

        assert!(message.is_none());
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].message.contains("charlie"));
        assert!(notifications[0].message.contains("1 country"));
        assert!(notifications[0].message.contains("3 words"));
    }

    #[test]
    fn test_send_note_single_word() {
        let config = make_test_config();
        let mut room = ChatRoom::new("test_room".to_string(), config);

        let user_id = "diana".to_string();
        let country = "D".to_string();
        room.add_participant(user_id.clone(), country.clone());

        // Send note with single word
        let mut note_map = HashMap::new();
        note_map.insert("A".to_string(), vec!["freedom".to_string()]);

        let action = UserAction::SendNote(note_map);
        let (message, notifications) = room.process_action(&user_id, &country, action);

        assert!(message.is_none());
        assert_eq!(notifications.len(), 1);
        assert!(notifications[0].message.contains("diana"));
        assert!(notifications[0].message.contains("1 country"));
        assert!(notifications[0].message.contains("1 word"));
    }

    #[test]
    fn test_send_note_updates_latest() {
        let config = make_test_config();
        let mut room = ChatRoom::new("test_room".to_string(), config);

        let user_id = "alice".to_string();
        let country = "A".to_string();
        room.add_participant(user_id.clone(), country.clone());

        // Send first note
        let mut note_map1 = HashMap::new();
        note_map1.insert("A".to_string(), vec!["freedom".to_string()]);
        let action1 = UserAction::SendNote(note_map1.clone());
        room.process_action(&user_id, &country, action1);

        // Verify first note is stored
        assert_eq!(room.get_player_note(&user_id).unwrap(), &note_map1);

        // Send second note (should replace first)
        let mut note_map2 = HashMap::new();
        note_map2.insert(
            "B".to_string(),
            vec!["monarchy".to_string(), "tradition".to_string()],
        );
        let action2 = UserAction::SendNote(note_map2.clone());
        room.process_action(&user_id, &country, action2);

        // Verify only latest note is stored
        let stored_note = room.get_player_note(&user_id).unwrap();
        assert_eq!(stored_note, &note_map2);
        assert_ne!(stored_note, &note_map1);

        // Verify we only have one entry per user
        assert_eq!(room.get_player_notes().len(), 1);
    }

    /// Creates a ChatRoom with controlled censorship settings for testing.
    /// Uses a simple filter config where:
    /// - Country "A" has "freedom" banned
    /// - Country "B" has "monarchy" banned
    fn make_censorship_test_room() -> ChatRoom {
        let config = make_test_config();
        let room = ChatRoom {
            room_id: "test_room".to_string(),
            config,
            participants: Vec::new(),
            messages: Vec::new(),
            message_counter: 0,
            filter: CensorshipFilter::new(config),
            allowed_words: vec![
                "hello".to_string(),
                "freedom".to_string(),
                "monarchy".to_string(),
                "world".to_string(),
            ],
            sender_censor: false,
            receiver_censor: true,
            shadow_ban: true,
            allowed: HashSet::new(),
            player_notes: HashMap::new(),
        };
        room
    }

    #[test]
    fn test_receiver_censorship_censors_banned_words() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = true;
        room.sender_censor = false;
        room.shadow_ban = false;

        // User from country C sends "freedom" (banned in A)
        room.add_participant("alice".to_string(), "C".to_string());
        let action = UserAction::SendMessage("freedom".to_string());
        let (msg, _) = room.process_action(&"alice".to_string(), &"C".to_string(), action);
        let message = msg.unwrap();

        // When viewed by country A, "freedom" should be censored
        let censored = room.censor_message_for(&message, &"A".to_string());
        assert!(censored.was_censored);
        assert_eq!(censored.content, "***");

        // When viewed by country B, "freedom" should NOT be censored
        let uncensored = room.censor_message_for(&message, &"B".to_string());
        assert!(!uncensored.was_censored);
        assert_eq!(uncensored.content, "freedom");
    }

    #[test]
    fn test_sender_censorship_censors_based_on_sender_country() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = false;
        room.sender_censor = true;
        room.shadow_ban = false;

        // User from country A sends "freedom" (banned in A)
        room.add_participant("alice".to_string(), "A".to_string());
        let action = UserAction::SendMessage("freedom".to_string());
        let (msg, _) = room.process_action(&"alice".to_string(), &"A".to_string(), action);
        let message = msg.unwrap();

        // When viewed by anyone, "freedom" should be censored because sender is from A
        let censored = room.censor_message_for(&message, &"C".to_string());
        assert!(censored.was_censored);
        assert_eq!(censored.content, "***");

        // User from country B sends "freedom" (not banned in B)
        let action2 = UserAction::SendMessage("freedom".to_string());
        let (msg2, _) = room.process_action(&"alice".to_string(), &"B".to_string(), action2);
        let message2 = msg2.unwrap();

        // Should NOT be censored because sender B doesn't have "freedom" banned
        let uncensored = room.censor_message_for(&message2, &"C".to_string());
        assert!(!uncensored.was_censored);
        assert_eq!(uncensored.content, "freedom");
    }

    #[test]
    fn test_shadow_ban_shows_own_messages_uncensored() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = true;
        room.sender_censor = false;
        room.shadow_ban = true;

        // User from country A sends "freedom" (banned in A)
        room.add_participant("alice".to_string(), "A".to_string());
        let action = UserAction::SendMessage("freedom".to_string());
        let (msg, _) = room.process_action(&"alice".to_string(), &"A".to_string(), action);
        let message = msg.unwrap();

        // When viewed by the sender (country A), message should be uncensored
        let own_view = room.censor_message_for(&message, &"A".to_string());
        assert!(!own_view.was_censored);
        assert_eq!(own_view.content, "freedom");

        // When viewed by another country A user, it would normally be censored
        // but shadow_ban only applies when sender_country == viewer_country
        // So another A viewer sees it censored
        // Actually, let's test with shadow_ban = false to see the difference
        room.shadow_ban = false;
        let other_view = room.censor_message_for(&message, &"A".to_string());
        assert!(other_view.was_censored);
        assert_eq!(other_view.content, "***");
    }

    #[test]
    fn test_allowed_countries_bypass_censorship() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = true;
        room.sender_censor = true;
        room.shadow_ban = false;
        room.allowed.insert("A".to_string());

        // User from country A sends "freedom" (banned in A, but A is in allowed)
        room.add_participant("alice".to_string(), "A".to_string());
        let action = UserAction::SendMessage("freedom".to_string());
        let (msg, _) = room.process_action(&"alice".to_string(), &"A".to_string(), action);
        let message = msg.unwrap();

        // Sender censorship should be bypassed for country A
        let view_by_b = room.censor_message_for(&message, &"B".to_string());
        assert!(!view_by_b.was_censored);

        // Receiver censorship should also be bypassed when viewer is A
        let view_by_a = room.censor_message_for(&message, &"A".to_string());
        assert!(!view_by_a.was_censored);
    }

    #[test]
    fn test_get_censored_state_applies_per_country_view() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = true;
        room.sender_censor = false;
        room.shadow_ban = false;

        room.add_participant("alice".to_string(), "C".to_string());
        room.add_participant("bob".to_string(), "A".to_string());

        // Send messages with words banned in different countries
        let action1 = UserAction::SendMessage("freedom".to_string()); // banned in A
        room.process_action(&"alice".to_string(), &"C".to_string(), action1);

        let action2 = UserAction::SendMessage("monarchy".to_string()); // banned in B
        room.process_action(&"alice".to_string(), &"C".to_string(), action2);

        // Get state for country A - "freedom" should be censored
        let state_a = room.get_censored_state_for(&"A".to_string());
        assert_eq!(state_a.recent_messages.len(), 2);
        assert_eq!(state_a.recent_messages[0].content, "***"); // freedom censored
        assert_eq!(state_a.recent_messages[1].content, "monarchy"); // monarchy visible

        // Get state for country B - "monarchy" should be censored
        let state_b = room.get_censored_state_for(&"B".to_string());
        assert_eq!(state_b.recent_messages[0].content, "freedom"); // freedom visible
        assert_eq!(state_b.recent_messages[1].content, "***"); // monarchy censored

        // Get state for country C - nothing censored (no banned words for C)
        let state_c = room.get_censored_state_for(&"C".to_string());
        assert_eq!(state_c.recent_messages[0].content, "freedom");
        assert_eq!(state_c.recent_messages[1].content, "monarchy");
    }

    #[test]
    fn test_combined_sender_and_receiver_censorship() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = true;
        room.sender_censor = true;
        room.shadow_ban = false;

        room.add_participant("alice".to_string(), "A".to_string());

        // User from A sends "monarchy" (banned in B, not A)
        let action = UserAction::SendMessage("monarchy".to_string());
        let (msg, _) = room.process_action(&"alice".to_string(), &"A".to_string(), action);
        let message = msg.unwrap();

        // Viewed by B: receiver_censor applies (monarchy banned in B)
        let view_b = room.censor_message_for(&message, &"B".to_string());
        assert!(view_b.was_censored);

        // Viewed by C: no censorship (not banned for sender A or receiver C)
        let view_c = room.censor_message_for(&message, &"C".to_string());
        assert!(!view_c.was_censored);
    }

    #[test]
    fn test_no_censorship_when_both_flags_disabled() {
        let mut room = make_censorship_test_room();
        room.receiver_censor = false;
        room.sender_censor = false;
        room.shadow_ban = false;

        room.add_participant("alice".to_string(), "A".to_string());

        // User from A sends "freedom" (normally banned in A)
        let action = UserAction::SendMessage("freedom".to_string());
        let (msg, _) = room.process_action(&"alice".to_string(), &"A".to_string(), action);
        let message = msg.unwrap();

        // Should not be censored for anyone
        let view_a = room.censor_message_for(&message, &"A".to_string());
        assert!(!view_a.was_censored);
        assert_eq!(view_a.content, "freedom");

        let view_b = room.censor_message_for(&message, &"B".to_string());
        assert!(!view_b.was_censored);
        assert_eq!(view_b.content, "freedom");
    }
}
