use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Words {
    pub normal: Vec<String>,
    pub censored: Vec<Vec<String>>,
}

pub fn load_words(path: &str) -> Words {
    let data = fs::read_to_string(path).expect("Failed to read words.json");
    serde_json::from_str(&data).expect("Failed to parse words.json")
}

/// Returns (allowed_words, banned_map) where banned_map: country_code -> Vec<String>
pub fn generate_allowed_and_banned_words(
    words: &Words,
    country_codes: &[&str],
) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let mut rng = rand::rng();
    // 1. Pick one word from each censored group
    let mut complex_words: Vec<String> = words
        .censored
        .iter()
        .map(|group| group.choose(&mut rng).unwrap().clone())
        .collect();
    // 2. allowed_words = normal + complex_words
    let mut allowed_words = words.normal.clone();
    allowed_words.extend(complex_words.iter().cloned());
    // 3. Pick 4 complex words for 4 countries as banned
    complex_words.shuffle(&mut rng);
    let mut banned_map = HashMap::new();
    for (i, &country) in country_codes.iter().enumerate().take(4) {
        banned_map.insert(country.to_string(), vec![complex_words[i].clone()]);
    }
    (allowed_words, banned_map)
}
