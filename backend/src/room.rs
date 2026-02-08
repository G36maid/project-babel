use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{debug, trace};

use crate::data::*;
use crate::game::{CensorshipGame, GameRules};

/// Game instructions message displayed when a room is created
const GAME_INSTRUCTIONS: &str = "Welcome to Project Babel! You are trying to communicate across a censorship firewall. Each country has different words that are banned. Work together to discover which words are censored for each country using the allowed symbols. Good luck!";

/// A chat room that manages participants, messages, and delegates game mechanics.
///
/// `ChatRoom` handles real-time communication between players while delegating
/// game-specific logic (censorship, victory conditions, word validation) to a
/// `GameRules` implementation.
///
/// # Example
///
/// ```ignore
/// let game = CensorshipGame::new(&config);
/// let room = ChatRoom::new("room_1".to_string(), game);
/// room.add_participant("alice".to_string(), "US".to_string());
/// let (msg, notifs) = room.process_action(&"alice", &"US", UserAction::SendMessage("hello".into()));
/// ```
pub struct ChatRoom {
    /// Unique identifier for this room.
    room_id: RoomId,
    /// List of participants currently in the room.
    participants: Vec<Participant>,
    /// All messages sent in this room.
    pub(crate) messages: Vec<Message>,
    /// Counter for generating unique message IDs.
    pub(crate) message_counter: MessageId,
    /// Game rules implementation that handles game-specific logic.
    game: Box<dyn GameRules>,
}

impl ChatRoom {
    pub fn new(room_id: RoomId, config: &'static FilterConfig) -> Self {
        let game = CensorshipGame::new(config);

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
            participants: Vec::new(),
            messages: vec![game_instructions],
            message_counter: 1,
            game: Box::new(game),
        }
    }

    pub fn win(&mut self) {
        let msg = "[SYSTEM] Censorship puzzle is finished!".to_string();
        self.message_counter += 1;
        self.messages.push(Message {
            id: self.message_counter,
            sender_id: "SYSTEM".to_string(),
            sender_country: "".to_string(),
            content: msg,
            timestamp: Self::current_timestamp(),
        });
        self.game.unlock_all_countries();
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
                    .filter(|w| self.game.is_word_allowed(w))
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
            UserAction::SubmitNotes(note_map) => {
                // Send note: players share their hypotheses about banned words
                // Store the latest notes for this user
                self.game.submit_player_notes(user_id, note_map.clone());

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
                    .filter(|w| self.game.is_word_allowed(w))
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

        let (content, was_censored) = self.game.censor_message_for(message, country);

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

    pub fn get_player_notes(
        &self,
    ) -> &std::collections::HashMap<UserId, std::collections::HashMap<CountryCode, Vec<String>>>
    {
        self.game.get_all_player_notes()
    }

    pub fn get_player_note(
        &self,
        user_id: &UserId,
    ) -> Option<&std::collections::HashMap<CountryCode, Vec<String>>> {
        self.game.get_player_notes(user_id)
    }

    /// Calculate player progress for all participants
    pub fn get_player_progress(&self) -> Vec<crate::data::PlayerProgress> {
        self.game.calculate_player_progress(&self.participants)
    }

    /// Check if all players have discovered all banned words
    pub fn check_victory(&mut self) -> bool {
        self.game.check_victory(&self.participants)
    }

    /// Get current victory state
    pub fn get_victory_state(&self) -> crate::data::VictoryState {
        self.game.get_victory_state(&self.participants)
    }

    /// Get the filter config from the game
    pub fn filter_config(&self) -> &FilterConfig {
        self.game.filter_config()
    }

    /// Get the allowed words for this room
    pub fn allowed_words(&self) -> &[String] {
        self.game.allowed_words()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CensorshipGame;
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

        let action = UserAction::SubmitNotes(note_map.clone());
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
        let action = UserAction::SubmitNotes(note_map);
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

        let action = UserAction::SubmitNotes(note_map);
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

        let action = UserAction::SubmitNotes(note_map);
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
        let action1 = UserAction::SubmitNotes(note_map1.clone());
        room.process_action(&user_id, &country, action1);

        // Verify first note is stored
        assert_eq!(room.get_player_note(&user_id).unwrap(), &note_map1);

        // Send second note (should replace first)
        let mut note_map2 = HashMap::new();
        note_map2.insert(
            "B".to_string(),
            vec!["monarchy".to_string(), "tradition".to_string()],
        );
        let action2 = UserAction::SubmitNotes(note_map2.clone());
        room.process_action(&user_id, &country, action2);

        // Verify only latest note is stored
        let stored_note = room.get_player_note(&user_id).unwrap();
        assert_eq!(stored_note, &note_map2);
        assert_ne!(stored_note, &note_map1);

        // Verify we only have one entry per user
        assert_eq!(room.get_player_notes().len(), 1);
    }
}
