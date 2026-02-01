import gleam/dict.{type Dict}
import gleam/dynamic/decode
import gleam/int
import gleam/json
import gleam/list
import simplifile

pub type Words {
  Words(normal: List(String), censored: List(List(String)))
}

/// Load words from JSON file
pub fn load_words(path: String) -> Result(Words, String) {
  case simplifile.read(path) {
    Ok(content) -> parse_words_json(content)
    Error(_) -> Error("Failed to read words.json")
  }
}

fn parse_words_json(content: String) -> Result(Words, String) {
  // Decode JSON structure: {"normal": [...], "censored": [[...], ...]}
  let decoder = {
    use normal <- decode.field("normal", decode.list(decode.string))
    use censored <- decode.field("censored", decode.list(decode.list(decode.string)))
    decode.success(Words(normal:, censored:))
  }

  case json.parse(content, decoder) {
    Ok(words) -> Ok(words)
    Error(_) -> Error("Failed to parse words.json")
  }
}

/// Get element at index from a list
fn list_at(items: List(a), index: Int) -> Result(a, Nil) {
  case index < 0 {
    True -> Error(Nil)
    False -> do_list_at(items, index)
  }
}

fn do_list_at(items: List(a), index: Int) -> Result(a, Nil) {
  case items {
    [] -> Error(Nil)
    [first, ..rest] -> case index {
      0 -> Ok(first)
      _ -> do_list_at(rest, index - 1)
    }
  }
}

/// Generate allowed words and banned words map for countries
/// Returns tuple of (allowed_words, banned_map) where banned_map: country -> [banned_words]
pub fn generate_allowed_and_banned_words(
  words: Words,
  country_codes: List(String),
  random_seed: Int,
) -> #(List(String), Dict(String, List(String))) {
  // 1. Pick one word from each censored group (using deterministic selection based on seed)
  let complex_words = list.index_map(words.censored, fn(group, idx) {
    let pick_idx = { random_seed + idx } % list.length(group)
    case list_at(group, pick_idx) {
      Ok(word) -> word
      Error(_) -> ""
    }
  })
  |> list.filter(fn(w) { w != "" })

  // 2. allowed_words = normal + complex_words
  let allowed_words = list.append(words.normal, complex_words)

  // 3. Shuffle complex words and assign first 4 to 4 countries
  let shuffled = shuffle_with_seed(complex_words, random_seed)

  // 4. Create banned_map
  let banned_map = list.index_fold(country_codes, dict.new(), fn(acc, country, idx) {
    case list_at(shuffled, idx) {
      Ok(word) -> dict.insert(acc, country, [word])
      Error(_) -> acc
    }
  })
  |> fn(d) {
    // Only take first 4 countries
    list.take(country_codes, 4)
    |> list.fold(dict.new(), fn(acc, country) {
      case dict.get(d, country) {
        Ok(ws) -> dict.insert(acc, country, ws)
        Error(_) -> acc
      }
    })
  }

  #(allowed_words, banned_map)
}

/// Simple deterministic shuffle using seed
fn shuffle_with_seed(items: List(a), seed: Int) -> List(a) {
  items
  |> list.index_map(fn(item, idx) { #(item, { seed * { idx + 1 } * 31 } % 1000) })
  |> list.sort(fn(a, b) { int.compare(a.1, b.1) })
  |> list.map(fn(pair) { pair.0 })
}
