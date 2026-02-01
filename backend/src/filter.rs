use crate::data::{CENSORSHIP_REPLACEMENT, CountryCode, FilterConfig};

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
        let mut result = content.to_string();
        let mut was_censored = false;

        // Apply sender's country filter if enabled
        if let (Some(_sender), Some(banned_words)) = (sender_country, self.config.banned_words.get(sender_country.unwrap_or(&"".to_string()))) {
            for word in banned_words {
                if result.to_lowercase().contains(&word.to_lowercase()) {
                    result = self.replace_word(&result, word);
                    was_censored = true;
                }
            }
        }

        // Apply receiver's country filter if enabled
        if let (Some(_receiver), Some(banned_words)) = (receiver_country, self.config.banned_words.get(receiver_country.unwrap_or(&"".to_string()))) {
            for word in banned_words {
                if result.to_lowercase().contains(&word.to_lowercase()) {
                    result = self.replace_word(&result, word);
                    was_censored = true;
                }
            }
        }

        (result, was_censored)
    }

    fn replace_word(&self, content: &str, word: &str) -> String {
        let lower_content = content.to_lowercase();
        let lower_word = word.to_lowercase();

        let mut result = String::new();
        let mut last_end = 0;

        for (start, _) in lower_content.match_indices(&lower_word) {
            result.push_str(&content[last_end..start]);
            result.push_str(CENSORSHIP_REPLACEMENT);
            last_end = start + word.len();
        }
        result.push_str(&content[last_end..]);

        result
    }
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
