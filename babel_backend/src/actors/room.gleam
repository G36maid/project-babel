import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/otp/actor
import gleam/set.{type Set}
import gleam/string
import logic/filter
import logic/words
import types

const game_instructions = "Welcome to Project Babel! You are trying to communicate across a censorship firewall. Each country has different words that are banned. Work together to discover which words are censored for each country using the allowed symbols. Good luck!"

/// Internal room state held by the actor
pub type InternalRoomState {
  InternalRoomState(
    room_id: String,
    participants: List(types.Participant),
    messages: List(types.Message),
    message_counter: Int,
    banned_words: Dict(String, List(String)),
    allowed_words: List(String),
    sender_censor: Bool,
    receiver_censor: Bool,
    shadow_ban: Bool,
    allowed_countries: Set(String),
    player_notes: Dict(String, Dict(String, List(String))),
    victory_achieved: Bool,
    victory_timestamp: Option(Int),
    subscribers: List(Subject(types.RoomUpdate)),
  )
}

/// Messages the Room actor can receive
pub type RoomMessage {
  Join(user_id: String, country: String, reply_to: Subject(Result(Nil, String)))
  ProcessAction(user_id: String, country: String, action: types.UserAction)
  Leave(user_id: String)
  GetState(country: String, reply_to: Subject(types.RoomState))
  GetInfo(reply_to: Subject(#(List(String), Dict(String, List(String)))))
  Subscribe(subscriber: Subject(types.RoomUpdate))
  Unsubscribe(subscriber: Subject(types.RoomUpdate))
  GetVictoryState(reply_to: Subject(types.VictoryState))
  CheckVictory(reply_to: Subject(Bool))
  Win
}

/// Start a new room actor
pub fn start(room_id: String) -> Result(Subject(RoomMessage), actor.StartError) {
  let state = init_state(room_id)
  let builder = actor.new(state)
  |> actor.on_message(fn(state, msg) { handle_message(msg, state) })
  case actor.start(builder) {
    Ok(started) -> Ok(started.data)
    Error(e) -> Error(e)
  }
}

fn init_state(room_id: String) -> InternalRoomState {
  // Load words and generate allowed/banned
  let words_result = words.load_words("words.json")
  let #(allowed_words, banned_words) = case words_result {
    Ok(w) -> {
      let seed = erlang_system_time()
      words.generate_allowed_and_banned_words(w, ["A", "B", "C", "D"], seed)
    }
    Error(_) -> #([], dict.new())
  }

  // Create initial game instructions message
  let instructions_msg = types.Message(
    id: 1,
    sender_id: "SYSTEM",
    sender_country: "",
    content: game_instructions,
    timestamp: current_timestamp(),
  )

  InternalRoomState(
    room_id: room_id,
    participants: [],
    messages: [instructions_msg],
    message_counter: 1,
    banned_words: banned_words,
    allowed_words: allowed_words,
    sender_censor: False,
    receiver_censor: True,
    shadow_ban: True,
    allowed_countries: set.new(),
    player_notes: dict.new(),
    victory_achieved: False,
    victory_timestamp: None,
    subscribers: [],
  )
}

fn handle_message(msg: RoomMessage, state: InternalRoomState) -> actor.Next(InternalRoomState, RoomMessage) {
  case msg {
    Join(user_id, country, reply_to) -> {
      case list.find(state.participants, fn(p) { p.user_id == user_id }) {
        Ok(_) -> {
          process.send(reply_to, Error("Already in room"))
          actor.continue(state)
        }
        Error(_) -> {
          let participant = types.Participant(
            user_id: user_id,
            country: country,
            joined_at: current_timestamp(),
          )
          let new_state = InternalRoomState(..state,
            participants: [participant, ..state.participants]
          )
          // Broadcast join notification
          let notification = types.Notification(message: user_id <> " joined the room")
          broadcast_update(new_state, [], [notification])
          process.send(reply_to, Ok(Nil))
          actor.continue(new_state)
        }
      }
    }

    ProcessAction(user_id, country, action) -> {
      let #(new_state, maybe_msg, notifications) = process_action(state, user_id, country, action)

      // Check victory after action
      let #(final_state, victory_notifications) = case check_victory(new_state) {
        True -> {
          let vs = InternalRoomState(..new_state,
            victory_achieved: True,
            victory_timestamp: Some(current_timestamp()),
          )
          #(vs, [types.Notification(message: "Victory! All players discovered all banned words!")])
        }
        False -> #(new_state, [])
      }

      // Broadcast update
      let new_messages = case maybe_msg {
        Some(m) -> [m]
        None -> []
      }
      broadcast_update(final_state, new_messages, list.append(notifications, victory_notifications))
      actor.continue(final_state)
    }

    Leave(user_id) -> {
      let new_participants = list.filter(state.participants, fn(p) { p.user_id != user_id })
      let new_state = InternalRoomState(..state, participants: new_participants)
      let notification = types.Notification(message: user_id <> " left the room")
      broadcast_update(new_state, [], [notification])
      actor.continue(new_state)
    }

    GetState(country, reply_to) -> {
      let room_state = get_censored_state_for(state, country)
      process.send(reply_to, room_state)
      actor.continue(state)
    }

    GetInfo(reply_to) -> {
      process.send(reply_to, #(state.allowed_words, state.banned_words))
      actor.continue(state)
    }

    Subscribe(subscriber) -> {
      let new_state = InternalRoomState(..state, subscribers: [subscriber, ..state.subscribers])
      actor.continue(new_state)
    }

    Unsubscribe(subscriber) -> {
      let new_subs = list.filter(state.subscribers, fn(s) { s != subscriber })
      let new_state = InternalRoomState(..state, subscribers: new_subs)
      actor.continue(new_state)
    }

    GetVictoryState(reply_to) -> {
      let vs = get_victory_state(state)
      process.send(reply_to, vs)
      actor.continue(state)
    }

    CheckVictory(reply_to) -> {
      process.send(reply_to, state.victory_achieved)
      actor.continue(state)
    }

    Win -> {
      // Add system message and unlock all countries
      let new_counter = state.message_counter + 1
      let win_msg = types.Message(
        id: new_counter,
        sender_id: "SYSTEM",
        sender_country: "",
        content: "[SYSTEM] Censorship puzzle is finished!",
        timestamp: current_timestamp(),
      )
      let all_countries = dict.keys(state.banned_words) |> set.from_list
      let new_state = InternalRoomState(..state,
        message_counter: new_counter,
        messages: [win_msg, ..state.messages],
        allowed_countries: all_countries,
        victory_achieved: True,
        victory_timestamp: Some(current_timestamp()),
      )
      broadcast_update(new_state, [win_msg], [])
      actor.continue(new_state)
    }
  }
}

fn process_action(state: InternalRoomState, user_id: String, country: String, action: types.UserAction)
  -> #(InternalRoomState, Option(types.Message), List(types.Notification)) {
  case action {
    types.SendMessageArray(words) -> {
      // Filter to only allowed words
      let filtered = list.filter(words, fn(w) { list.contains(state.allowed_words, w) })
      let content = string.join(filtered, " ")
      case content {
        "" -> #(state, None, [])
        _ -> {
          let new_counter = state.message_counter + 1
          let msg = types.Message(
            id: new_counter,
            sender_id: user_id,
            sender_country: country,
            content: content,
            timestamp: current_timestamp(),
          )
          let new_state = InternalRoomState(..state,
            message_counter: new_counter,
            messages: [msg, ..state.messages],
          )
          #(new_state, Some(msg), [])
        }
      }
    }

    types.SendMessage(content) -> {
      // Fallback: split and filter
      let words = string.split(content, " ")
      let filtered = list.filter(words, fn(w) { list.contains(state.allowed_words, w) })
      let filtered_content = string.join(filtered, " ")
      case filtered_content {
        "" -> #(state, None, [])
        _ -> {
          let new_counter = state.message_counter + 1
          let msg = types.Message(
            id: new_counter,
            sender_id: user_id,
            sender_country: country,
            content: filtered_content,
            timestamp: current_timestamp(),
          )
          let new_state = InternalRoomState(..state,
            message_counter: new_counter,
            messages: [msg, ..state.messages],
          )
          #(new_state, Some(msg), [])
        }
      }
    }

    types.SubmitNotes(notes) -> {
      // Store player's notes
      let new_notes = dict.insert(state.player_notes, user_id, notes)
      let new_state = InternalRoomState(..state, player_notes: new_notes)

      // Generate notification
      let country_count = dict.size(notes)
      let total_words = dict.values(notes) |> list.fold(0, fn(acc, ws) { acc + list.length(ws) })
      let country_label = case country_count { 1 -> "country" _ -> "countries" }
      let word_label = case total_words { 1 -> "word" _ -> "words" }
      let notification = types.Notification(
        message: user_id <> " shared exploration notes ("
          <> int.to_string(country_count) <> " " <> country_label <> ", "
          <> int.to_string(total_words) <> " " <> word_label <> ")"
      )
      #(new_state, None, [notification])
    }

    types.LeaveRoom -> {
      let new_participants = list.filter(state.participants, fn(p) { p.user_id != user_id })
      let new_state = InternalRoomState(..state, participants: new_participants)
      let notification = types.Notification(message: user_id <> " left the room")
      #(new_state, None, [notification])
    }
  }
}

fn get_censored_state_for(state: InternalRoomState, country: String) -> types.RoomState {
  let censored_messages = list.map(state.messages, fn(msg) {
    censor_message_for(state, msg, country)
  })
  |> list.reverse

  types.RoomState(
    room_id: state.room_id,
    participants: state.participants,
    recent_messages: censored_messages,
  )
}

fn censor_message_for(state: InternalRoomState, msg: types.Message, viewer_country: String) -> types.CensoredMessage {
  // System messages never censored
  case msg.sender_id {
    "SYSTEM" -> types.CensoredMessage(
      id: msg.id,
      sender_id: msg.sender_id,
      content: msg.content,
      was_censored: False,
    )
    _ -> {
      // Shadow ban: sender sees own message uncensored
      case state.shadow_ban && msg.sender_country == viewer_country {
        True -> types.CensoredMessage(
          id: msg.id,
          sender_id: msg.sender_id,
          content: msg.content,
          was_censored: False,
        )
        False -> {
          let sender = case state.sender_censor && !set.contains(state.allowed_countries, msg.sender_country) {
            True -> Some(msg.sender_country)
            False -> None
          }
          let receiver = case state.receiver_censor && !set.contains(state.allowed_countries, viewer_country) {
            True -> Some(viewer_country)
            False -> None
          }
          let #(content, was_censored) = filter.censor_message(msg.content, sender, receiver, state.banned_words)
          types.CensoredMessage(
            id: msg.id,
            sender_id: msg.sender_id,
            content: content,
            was_censored: was_censored,
          )
        }
      }
    }
  }
}

fn check_victory(state: InternalRoomState) -> Bool {
  case state.victory_achieved {
    True -> True
    False -> {
      case list.is_empty(state.participants) {
        True -> False
        False -> {
          let progress = get_player_progress_list(state)
          list.all(progress, fn(p) { p.completed })
        }
      }
    }
  }
}

fn get_victory_state(state: InternalRoomState) -> types.VictoryState {
  types.VictoryState(
    achieved: state.victory_achieved,
    player_progress: get_player_progress_list(state),
    unlocked_at: state.victory_timestamp,
  )
}

fn get_player_progress_list(state: InternalRoomState) -> List(types.PlayerProgress) {
  // Get countries of current participants
  let active_countries = list.map(state.participants, fn(p) { p.country }) |> set.from_list

  // Get banned words for active countries
  let all_banned = dict.to_list(state.banned_words)
    |> list.filter(fn(pair) { set.contains(active_countries, pair.0) })
    |> list.flat_map(fn(pair) { pair.1 })
    |> list.map(string.lowercase)
    |> set.from_list

  let total_required = set.size(all_banned)

  list.map(state.participants, fn(participant) {
    let discovered_count = case dict.get(state.player_notes, participant.user_id) {
      Ok(notes) -> {
        let discovered = dict.values(notes)
          |> list.flatten
          |> list.map(string.lowercase)
          |> set.from_list
        set.intersection(discovered, all_banned) |> set.size
      }
      Error(_) -> 0
    }

    types.PlayerProgress(
      user_id: participant.user_id,
      country: participant.country,
      discovered_count: discovered_count,
      total_required: total_required,
      completed: discovered_count >= total_required,
    )
  })
}

fn broadcast_update(state: InternalRoomState, new_messages: List(types.Message), notifications: List(types.Notification)) {
  let room_state = get_censored_state_for(state, "")
  let update = types.RoomUpdate(
    room_state: room_state,
    new_messages: new_messages,
    notifications: notifications,
    room_closed: False,
    victory: case state.victory_achieved {
      True -> Some(get_victory_state(state))
      False -> None
    },
  )
  list.each(state.subscribers, fn(sub) {
    process.send(sub, update)
  })
}

// External functions
@external(erlang, "erlang", "system_time")
fn erlang_system_time() -> Int

fn current_timestamp() -> Int {
  erlang_system_time() / 1_000_000_000
}
