import gleam/dict.{type Dict}
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string

const censorship_replacement = "***"

/// Apply censorship to a message based on sender and receiver countries
/// Returns tuple of (censored_content, was_censored)
pub fn censor_message(
  content: String,
  sender_country: Option(String),
  receiver_country: Option(String),
  banned_words: Dict(String, List(String)),
) -> #(String, Bool) {
  let #(result, was_censored) = case sender_country {
    Some(country) -> {
      case dict.get(banned_words, country) {
        Ok(words) -> apply_censorship(content, words)
        Error(_) -> #(content, False)
      }
    }
    None -> #(content, False)
  }

  // Apply receiver's country filter
  case receiver_country {
    Some(country) -> {
      case dict.get(banned_words, country) {
        Ok(words) -> {
          let #(new_result, censored) = apply_censorship(result, words)
          #(new_result, was_censored || censored)
        }
        Error(_) -> #(result, was_censored)
      }
    }
    None -> #(result, was_censored)
  }
}

/// Apply censorship for a list of banned words
fn apply_censorship(content: String, banned_words: List(String)) -> #(String, Bool) {
  list.fold(banned_words, #(content, False), fn(acc, word) {
    let #(current, was_censored) = acc
    let #(replaced, did_replace) = replace_word(current, word)
    #(replaced, was_censored || did_replace)
  })
}

/// Replace all case-insensitive occurrences of a word with ***
fn replace_word(content: String, word: String) -> #(String, Bool) {
  let lower_content = string.lowercase(content)
  let lower_word = string.lowercase(word)

  case string.contains(lower_content, lower_word) {
    True -> {
      // Find and replace all occurrences
      let replaced = do_replace(content, lower_content, lower_word, "", 0)
      #(replaced, True)
    }
    False -> #(content, False)
  }
}

/// Helper to replace all occurrences preserving case
fn do_replace(
  original: String,
  lower: String,
  word: String,
  acc: String,
  start: Int,
) -> String {
  case find_index(lower, word, start) {
    Some(idx) -> {
      let before = string.slice(original, start, idx - start)
      let new_start = idx + string.length(word)
      do_replace(original, lower, word, acc <> before <> censorship_replacement, new_start)
    }
    None -> {
      acc <> string.drop_start(original, start)
    }
  }
}

/// Find index of substring starting from position
fn find_index(haystack: String, needle: String, start: Int) -> Option(Int) {
  let suffix = string.drop_start(haystack, start)
  case string.split_once(suffix, needle) {
    Ok(#(before, _)) -> Some(start + string.length(before))
    Error(_) -> None
  }
}
