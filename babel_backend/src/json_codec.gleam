import gleam/dict.{type Dict}
import gleam/dynamic.{type Dynamic}
import gleam/dynamic/decode.{type DecodeError}
import gleam/json.{type Json}
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import types

// ============ ENCODERS ============

pub fn encode_message(msg: types.Message) -> Json {
  json.object([
    #("id", json.int(msg.id)),
    #("sender_id", json.string(msg.sender_id)),
    #("sender_country", json.string(msg.sender_country)),
    #("content", json.string(msg.content)),
    #("timestamp", json.int(msg.timestamp)),
  ])
}

pub fn encode_censored_message(msg: types.CensoredMessage) -> Json {
  json.object([
    #("id", json.int(msg.id)),
    #("sender_id", json.string(msg.sender_id)),
    #("content", json.string(msg.content)),
    #("was_censored", json.bool(msg.was_censored)),
  ])
}

pub fn encode_participant(p: types.Participant) -> Json {
  json.object([
    #("user_id", json.string(p.user_id)),
    #("country", json.string(p.country)),
    #("joined_at", json.int(p.joined_at)),
  ])
}

pub fn encode_notification(n: types.Notification) -> Json {
  json.object([#("message", json.string(n.message))])
}

pub fn encode_player_progress(pp: types.PlayerProgress) -> Json {
  json.object([
    #("user_id", json.string(pp.user_id)),
    #("country", json.string(pp.country)),
    #("discovered_count", json.int(pp.discovered_count)),
    #("total_required", json.int(pp.total_required)),
    #("completed", json.bool(pp.completed)),
  ])
}

pub fn encode_victory_state(vs: types.VictoryState) -> Json {
  json.object([
    #("achieved", json.bool(vs.achieved)),
    #("player_progress", json.array(vs.player_progress, encode_player_progress)),
    #("unlocked_at", encode_optional_int(vs.unlocked_at)),
  ])
}

fn encode_optional_int(opt: Option(Int)) -> Json {
  case opt {
    Some(val) -> json.int(val)
    None -> json.null()
  }
}

fn encode_optional_victory_state(opt: Option(types.VictoryState)) -> Json {
  case opt {
    Some(vs) -> encode_victory_state(vs)
    None -> json.null()
  }
}

pub fn encode_room_state(rs: types.RoomState) -> Json {
  json.object([
    #("room_id", json.string(rs.room_id)),
    #("participants", json.array(rs.participants, encode_participant)),
    #("recent_messages", json.array(rs.recent_messages, encode_censored_message)),
  ])
}

pub fn encode_room_update(ru: types.RoomUpdate) -> Json {
  json.object([
    #("room_state", encode_room_state(ru.room_state)),
    #("new_messages", json.array(ru.new_messages, encode_message)),
    #("notifications", json.array(ru.notifications, encode_notification)),
    #("room_closed", json.bool(ru.room_closed)),
    #("victory", encode_optional_victory_state(ru.victory)),
  ])
}

pub fn encode_client_room_update(cru: types.ClientRoomUpdate) -> Json {
  json.object([
    #("room_state", encode_room_state(cru.room_state)),
    #("new_messages", json.array(cru.new_messages, encode_censored_message)),
    #("notifications", json.array(cru.notifications, encode_notification)),
    #("room_closed", json.bool(cru.room_closed)),
    #("victory", encode_optional_victory_state(cru.victory)),
  ])
}

pub fn encode_filter_config(fc: types.FilterConfig) -> Json {
  let entries = dict.to_list(fc.banned_words)
  json.object(
    list.map(entries, fn(entry) {
      let #(key, words) = entry
      #(key, json.array(words, json.string))
    }),
  )
}

// ============ DECODERS ============

/// Create a decoder for UserAction
pub fn make_user_action_decoder() -> decode.Decoder(types.UserAction) {
  let send_message_decoder = {
    use content <- decode.field("send_message", decode.string)
    decode.success(types.SendMessage(content))
  }

  let send_message_array_decoder = {
    use words <- decode.field("send_message_array", decode.list(decode.string))
    decode.success(types.SendMessageArray(words))
  }

  let submit_notes_decoder = {
    use notes <- decode.field("submit_notes", decode.dict(decode.string, decode.list(decode.string)))
    decode.success(types.SubmitNotes(notes))
  }

  let leave_room_decoder = {
    use _ <- decode.field("leave_room", decode.dynamic)
    decode.success(types.LeaveRoom)
  }

  // Use one_of to try each variant - first decoder plus list of alternatives
  decode.one_of(send_message_decoder, [
    send_message_array_decoder,
    submit_notes_decoder,
    leave_room_decoder,
  ])
}

/// Decode UserAction from JSON
/// Format: {"send_message": "content"} or {"send_message_array": ["a", "b"]}
/// or {"submit_notes": {"A": ["word"]}} or {"leave_room": null}
pub fn decode_user_action(data: Dynamic) -> Result(types.UserAction, List(DecodeError)) {
  // Try each variant
  let send_message_decoder = {
    use content <- decode.field("send_message", decode.string)
    decode.success(types.SendMessage(content))
  }

  let send_message_array_decoder = {
    use words <- decode.field("send_message_array", decode.list(decode.string))
    decode.success(types.SendMessageArray(words))
  }

  let submit_notes_decoder = {
    use notes <- decode.field("submit_notes", decode.dict(decode.string, decode.list(decode.string)))
    decode.success(types.SubmitNotes(notes))
  }

  let leave_room_decoder = {
    use _ <- decode.field("leave_room", decode.dynamic)
    decode.success(types.LeaveRoom)
  }

  // Try each decoder in order
  case decode.run(data, send_message_decoder) {
    Ok(action) -> Ok(action)
    Error(_) -> case decode.run(data, send_message_array_decoder) {
      Ok(action) -> Ok(action)
      Error(_) -> case decode.run(data, submit_notes_decoder) {
        Ok(action) -> Ok(action)
        Error(_) -> decode.run(data, leave_room_decoder)
      }
    }
  }
}

/// Decode login request: {"username": "...", "country": "..."}
pub fn decode_login_request(data: Dynamic) -> Result(#(String, String), List(DecodeError)) {
  let decoder = {
    use username <- decode.field("username", decode.string)
    use country <- decode.field("country", decode.string)
    decode.success(#(username, country))
  }
  decode.run(data, decoder)
}

/// Decode solve request: {"answer": {"A": ["word1"], "B": ["word2"], ...}}
pub fn decode_solve_request(data: Dynamic) -> Result(Dict(String, List(String)), List(DecodeError)) {
  let decoder = {
    use answer <- decode.field("answer", decode.dict(decode.string, decode.list(decode.string)))
    decode.success(answer)
  }
  decode.run(data, decoder)
}

/// Decode submit notes request: {"notes": {"A": ["word1"], "B": ["word2"]}}
pub fn decode_submit_notes_request(data: Dynamic) -> Result(Dict(String, List(String)), List(DecodeError)) {
  let decoder = {
    use notes <- decode.field("notes", decode.dict(decode.string, decode.list(decode.string)))
    decode.success(notes)
  }
  decode.run(data, decoder)
}
