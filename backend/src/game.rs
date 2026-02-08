use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::data::*;
use crate::filter::CensorshipFilter;
use crate::words::{generate_allowed_and_banned_words, load_words};

/// Trait defining game-specific mechanics separate from chat room management.
///
/// This trait encapsulates all game logic including censorship rules, word validation,
/// player progress tracking, and victory conditions. Implementations can define
/// different game variants while maintaining the same chat infrastructure.
pub trait GameRules: Send + Sync {
    /// Get the list of words players are allowed to use in messages.
    fn allowed_words(&self) -> &[String];

    /// Check if a word is in the allowed word list.
    fn is_word_allowed(&self, word: &str) -> bool;

    /// Apply censorship to a message for a specific viewer.
    /// Returns (censored_content, was_censored)
    fn censor_message_for(&self, message: &Message, viewer_country: &CountryCode)
    -> (String, bool);

    /// Store player notes (hypotheses about banned words).
    fn submit_player_notes(&mut self, user_id: &UserId, notes: HashMap<CountryCode, Vec<String>>);

    /// Get stored notes for a specific player.
    fn get_player_notes(&self, user_id: &UserId) -> Option<&HashMap<CountryCode, Vec<String>>>;

    /// Get all player notes.
    fn get_all_player_notes(&self) -> &HashMap<UserId, HashMap<CountryCode, Vec<String>>>;

    /// Calculate progress for all participants.
    fn calculate_player_progress(&self, participants: &[Participant]) -> Vec<PlayerProgress>;

    /// Check if victory conditions are met and update state.
    /// Returns true if victory was achieved.
    fn check_victory(&mut self, participants: &[Participant]) -> bool;

    /// Get the current victory state.
    fn get_victory_state(&self, participants: &[Participant]) -> VictoryState;

    /// Mark all countries as allowed (bypass censorship).
    fn unlock_all_countries(&mut self);

    /// Get the filter config for this game.
    fn filter_config(&self) -> &FilterConfig;
}

/// Implementation of censorship-based puzzle game rules.
///
/// This struct manages the word-guessing puzzle where players must discover
/// which words are banned in different countries by observing censorship patterns.
pub struct CensorshipGame {
    /// The censorship filter that processes messages based on country rules.
    filter: CensorshipFilter,
    /// Shared reference to the filter configuration.
    config: Arc<FilterConfig>,
    /// Words that participants are allowed to use in messages.
    allowed_words: Vec<String>,
    /// Whether to apply censorship based on the sender's country.
    sender_censor: bool,
    /// Whether to apply censorship based on the receiver's country.
    receiver_censor: bool,
    /// Whether users see their own messages uncensored (shadow ban mode).
    shadow_ban: bool,
    /// Countries that are exempt from censorship.
    allowed_countries: HashSet<String>,
    /// Player notes storing hypotheses about banned words per country.
    player_notes: HashMap<UserId, HashMap<CountryCode, Vec<String>>>,
    /// Whether victory has been achieved.
    victory_achieved: bool,
    /// Timestamp when victory was achieved.
    victory_timestamp: Option<Timestamp>,
}

impl CensorshipGame {
    /// Create a new censorship game with generated words and default settings.
    pub fn new(config: &FilterConfig) -> Self {
        // Load words from words.json
        let words = load_words("words.json");
        let country_codes = ["A", "B", "C", "D"];
        let (allowed_words, banned_map) = generate_allowed_and_banned_words(&words, &country_codes);

        // Clone and update the config's banned_words for this game
        let mut config_owned = config.clone();
        for (country, banned) in &banned_map {
            config_owned
                .banned_words
                .insert(country.clone(), banned.clone());
        }
        let config_arc = Arc::new(config_owned);

        Self {
            filter: CensorshipFilter::new(Arc::clone(&config_arc)),
            config: config_arc,
            allowed_words,
            sender_censor: true,
            receiver_censor: true,
            shadow_ban: false,
            allowed_countries: HashSet::new(),
            player_notes: HashMap::new(),
            victory_achieved: false,
            victory_timestamp: None,
        }
    }

    /// Create a game instance for testing with custom configuration.
    #[cfg(test)]
    pub fn new_for_test(
        config: Arc<FilterConfig>,
        allowed_words: Vec<String>,
        sender_censor: bool,
        receiver_censor: bool,
        shadow_ban: bool,
    ) -> Self {
        Self {
            filter: CensorshipFilter::new(Arc::clone(&config)),
            config,
            allowed_words,
            sender_censor,
            receiver_censor,
            shadow_ban,
            allowed_countries: HashSet::new(),
            player_notes: HashMap::new(),
            victory_achieved: false,
            victory_timestamp: None,
        }
    }

    fn current_timestamp() -> Timestamp {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl GameRules for CensorshipGame {
    fn allowed_words(&self) -> &[String] {
        &self.allowed_words
    }

    fn is_word_allowed(&self, word: &str) -> bool {
        self.allowed_words.contains(&word.to_string())
    }

    fn censor_message_for(
        &self,
        message: &Message,
        viewer_country: &CountryCode,
    ) -> (String, bool) {
        // System messages are never censored
        if message.sender_id == "SYSTEM" {
            return (message.content.clone(), false);
        }

        let sender =
            if self.sender_censor && !self.allowed_countries.contains(&message.sender_country) {
                Some(&message.sender_country)
            } else {
                None
            };

        let receiver = if self.receiver_censor && !self.allowed_countries.contains(viewer_country) {
            Some(viewer_country)
        } else {
            None
        };

        if self.shadow_ban && &message.sender_country == viewer_country {
            (message.content.clone(), false)
        } else {
            self.filter
                .censor_message(&message.content, sender, receiver)
        }
    }

    fn submit_player_notes(&mut self, user_id: &UserId, notes: HashMap<CountryCode, Vec<String>>) {
        self.player_notes.insert(user_id.clone(), notes);
    }

    fn get_player_notes(&self, user_id: &UserId) -> Option<&HashMap<CountryCode, Vec<String>>> {
        self.player_notes.get(user_id)
    }

    fn get_all_player_notes(&self) -> &HashMap<UserId, HashMap<CountryCode, Vec<String>>> {
        &self.player_notes
    }

    fn calculate_player_progress(&self, participants: &[Participant]) -> Vec<PlayerProgress> {
        // Get countries of current participants
        let active_countries: HashSet<String> =
            participants.iter().map(|p| p.country.clone()).collect();

        // Get only banned words for countries that are currently in the room
        let all_banned_words: HashSet<String> = self
            .config
            .banned_words
            .iter()
            .filter(|(country, _)| active_countries.contains(*country))
            .flat_map(|(_, words)| words)
            .map(|s| s.to_lowercase())
            .collect();

        let total_required = all_banned_words.len();

        participants
            .iter()
            .map(|participant| {
                let discovered_count =
                    if let Some(notes) = self.player_notes.get(&participant.user_id) {
                        // Collect all unique words from player's notes
                        let discovered: HashSet<String> =
                            notes.values().flatten().map(|s| s.to_lowercase()).collect();

                        // Count how many match actual banned words
                        discovered.intersection(&all_banned_words).count()
                    } else {
                        0
                    };

                PlayerProgress {
                    user_id: participant.user_id.clone(),
                    country: participant.country.clone(),
                    discovered_count,
                    total_required,
                    completed: discovered_count >= total_required,
                }
            })
            .collect()
    }

    fn check_victory(&mut self, participants: &[Participant]) -> bool {
        if self.victory_achieved {
            return true;
        }

        // Need at least one player
        if participants.is_empty() {
            return false;
        }

        let progress = self.calculate_player_progress(participants);

        // All players must have completed
        let all_completed = progress.iter().all(|p| p.completed);

        if all_completed {
            self.victory_achieved = true;
            self.victory_timestamp = Some(Self::current_timestamp());
        }

        all_completed
    }

    fn get_victory_state(&self, participants: &[Participant]) -> VictoryState {
        VictoryState {
            achieved: self.victory_achieved,
            player_progress: self.calculate_player_progress(participants),
            unlocked_at: self.victory_timestamp,
        }
    }

    fn unlock_all_countries(&mut self) {
        for country in self.config.banned_words.keys() {
            self.allowed_countries.insert(country.clone());
        }
    }

    fn filter_config(&self) -> &FilterConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config() -> Arc<FilterConfig> {
        let mut banned_words = HashMap::new();
        banned_words.insert("A".to_string(), vec!["freedom".to_string()]);
        banned_words.insert("B".to_string(), vec!["monarchy".to_string()]);
        Arc::new(FilterConfig { banned_words })
    }

    #[test]
    fn test_is_word_allowed() {
        let config = make_test_config();
        let game = CensorshipGame::new_for_test(
            config,
            vec!["hello".to_string(), "world".to_string()],
            false,
            false,
            false,
        );

        assert!(game.is_word_allowed("hello"));
        assert!(game.is_word_allowed("world"));
        assert!(!game.is_word_allowed("foo"));
    }

    #[test]
    fn test_submit_and_get_player_notes() {
        let config = make_test_config();
        let mut game = CensorshipGame::new_for_test(config, vec![], false, false, false);

        let user_id = "alice".to_string();
        let mut notes = HashMap::new();
        notes.insert("A".to_string(), vec!["freedom".to_string()]);

        game.submit_player_notes(&user_id, notes.clone());

        assert_eq!(game.get_player_notes(&user_id), Some(&notes));
        assert_eq!(game.get_all_player_notes().len(), 1);
    }

    #[test]
    fn test_calculate_player_progress() {
        let config = make_test_config();
        let mut game = CensorshipGame::new_for_test(config, vec![], false, false, false);

        let participants = vec![Participant {
            user_id: "alice".to_string(),
            country: "A".to_string(),
            joined_at: 0,
        }];

        // No notes submitted yet
        let progress = game.calculate_player_progress(&participants);
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].discovered_count, 0);
        assert_eq!(progress[0].total_required, 1); // Only "freedom" for country A

        // Submit correct note
        let mut notes = HashMap::new();
        notes.insert("A".to_string(), vec!["freedom".to_string()]);
        game.submit_player_notes(&"alice".to_string(), notes);

        let progress = game.calculate_player_progress(&participants);
        assert_eq!(progress[0].discovered_count, 1);
        assert!(progress[0].completed);
    }

    #[test]
    fn test_check_victory() {
        let config = make_test_config();
        let mut game = CensorshipGame::new_for_test(config, vec![], false, false, false);

        let participants = vec![Participant {
            user_id: "alice".to_string(),
            country: "A".to_string(),
            joined_at: 0,
        }];

        // Victory not achieved yet
        assert!(!game.check_victory(&participants));

        // Submit correct notes
        let mut notes = HashMap::new();
        notes.insert("A".to_string(), vec!["freedom".to_string()]);
        game.submit_player_notes(&"alice".to_string(), notes);

        // Victory achieved
        assert!(game.check_victory(&participants));

        // Subsequent calls should still return true
        assert!(game.check_victory(&participants));
    }

    #[test]
    fn test_censor_message_receiver_mode() {
        let config = make_test_config();
        let game = CensorshipGame::new_for_test(config, vec![], false, true, false);

        let message = Message {
            id: 1,
            sender_id: "alice".to_string(),
            sender_country: "C".to_string(),
            content: "freedom".to_string(),
            timestamp: 0,
        };

        // Viewed by country A (has "freedom" banned)
        let (content, was_censored) = game.censor_message_for(&message, &"A".to_string());
        assert!(was_censored);
        assert_eq!(content, "***");

        // Viewed by country B (doesn't have "freedom" banned)
        let (content, was_censored) = game.censor_message_for(&message, &"B".to_string());
        assert!(!was_censored);
        assert_eq!(content, "freedom");
    }

    #[test]
    fn test_unlock_all_countries() {
        let config = make_test_config();
        let mut game = CensorshipGame::new_for_test(config, vec![], true, true, false);

        let message = Message {
            id: 1,
            sender_id: "alice".to_string(),
            sender_country: "A".to_string(),
            content: "freedom".to_string(),
            timestamp: 0,
        };

        // Before unlock, censored
        let (_, was_censored) = game.censor_message_for(&message, &"B".to_string());
        assert!(was_censored);

        // After unlock, not censored
        game.unlock_all_countries();
        let (content, was_censored) = game.censor_message_for(&message, &"B".to_string());
        assert!(!was_censored);
        assert_eq!(content, "freedom");
    }
}
