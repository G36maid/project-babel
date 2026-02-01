<<<<<<< HEAD
use crate::data::{CountryCode, FilterConfig, CENSORSHIP_REPLACEMENT};
=======
use crate::data::{CENSORSHIP_REPLACEMENT, CountryCode, FilterConfig};
use std::collections::HashMap;
>>>>>>> 8b87053 (refactor(backend): unify censorship logic and eliminate clippy warnings)

pub struct CensorshipFilter {
    pub(crate) config: &'static FilterConfig,
}

impl CensorshipFilter {
    pub fn new(config: &'static FilterConfig) -> Self {
        Self { config }
    }

    /// Applies dual-filter censorship: sender's filter + receiver's filter, controlled by flags
    pub fn censor_message(
        &self,
        content: &str,
        sender_country: Option<&CountryCode>,
        receiver_country: Option<&CountryCode>,
    ) -> (String, bool) {
        apply_censorship(
            content,
            &self.config.banned_words,
            sender_country,
            receiver_country,
        )
    }
}

/// Unified censorship function: applies banned words from sender and receiver countries.
/// Pre-collects all applicable banned words to eliminate nested conditionals and optimize performance.
///
/// # Arguments
/// * `content` - The text to censor
/// * `banned_words_map` - Map of country code to list of banned words
/// * `sender_country` - Optional sender country (words banned in this country will be censored)
/// * `receiver_country` - Optional receiver country (words banned in this country will be censored)
///
/// # Returns
/// * Tuple of (censored_content, was_censored)
pub fn apply_censorship(
    content: &str,
    banned_words_map: &HashMap<CountryCode, Vec<String>>,
    sender_country: Option<&CountryCode>,
    receiver_country: Option<&CountryCode>,
) -> (String, bool) {
    // Pre-collect all applicable banned words to avoid nested conditionals
    let mut active_words = Vec::new();

    if let Some(country) = sender_country {
        if let Some(words) = banned_words_map.get(country) {
            active_words.extend(words.iter());
        }
    }

    if let Some(country) = receiver_country {
        if let Some(words) = banned_words_map.get(country) {
            active_words.extend(words.iter());
        }
    }

    // Early return if no censorship needed
    if active_words.is_empty() {
        return (content.to_string(), false);
    }

    // Perform censorship with optimized single-pass replacement
    let mut result = content.to_string();
    let mut was_censored = false;

    for word in active_words {
        let (new_result, replaced) = replace_word_case_insensitive(&result, word);
        if replaced {
            result = new_result;
            was_censored = true;
        }
    }

    (result, was_censored)
}

/// Case-insensitive word replacement helper.
/// Uses match_indices for efficient substring matching.
fn replace_word_case_insensitive(content: &str, word: &str) -> (String, bool) {
    let lower_content = content.to_lowercase();
    let lower_word = word.to_lowercase();

    if !lower_content.contains(&lower_word) {
        return (content.to_string(), false);
    }

    let mut result = String::with_capacity(content.len());
    let mut last_end = 0;

    for (start, _) in lower_content.match_indices(&lower_word) {
        result.push_str(&content[last_end..start]);
        result.push_str(CENSORSHIP_REPLACEMENT);
        last_end = start + word.len();
    }
    result.push_str(&content[last_end..]);

    (result, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config() -> FilterConfig {
        let mut banned_words = HashMap::new();
        banned_words.insert("A".to_string(), vec!["bad".to_string(), "evil".to_string()]);
        banned_words.insert("B".to_string(), vec!["wrong".to_string()]);

        FilterConfig { banned_words }
    }

    #[test]
    fn test_censor_sender_words() {
        let config = Box::leak(Box::new(make_config()));
        let filter = CensorshipFilter::new(config);

        let sender = "A".to_string();
        let receiver = "C".to_string();
        let (result, censored) =
            filter.censor_message("This is bad", Some(&sender), Some(&receiver));
        assert!(censored);
        assert_eq!(result, "This is ***");
    }

    #[test]
    fn test_censor_receiver_words() {
        let config = Box::leak(Box::new(make_config()));
        let filter = CensorshipFilter::new(config);

        let sender = "C".to_string();
        let receiver = "B".to_string();
        let (result, censored) =
            filter.censor_message("This is wrong", Some(&sender), Some(&receiver));
        assert!(censored);
        assert_eq!(result, "This is ***");
    }

    #[test]
    fn test_censor_both_filters() {
        let config = Box::leak(Box::new(make_config()));
        let filter = CensorshipFilter::new(config);

        let sender = "A".to_string();
        let receiver = "B".to_string();
        let (result, censored) =
            filter.censor_message("bad and wrong", Some(&sender), Some(&receiver));
        assert!(censored);
        assert_eq!(result, "*** and ***");
    }

    #[test]
    fn test_no_censorship() {
        let config = Box::leak(Box::new(make_config()));
        let filter = CensorshipFilter::new(config);

        let sender = "A".to_string();
        let receiver = "B".to_string();
        let (result, censored) =
            filter.censor_message("Hello world", Some(&sender), Some(&receiver));
        assert!(!censored);
        assert_eq!(result, "Hello world");
    }
}
