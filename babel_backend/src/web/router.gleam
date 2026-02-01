import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/http.{Get, Post}
import gleam/json
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/set
import wisp.{type Request, type Response}
import actors/manager.{type ManagerMessage}
import actors/room.{type RoomMessage}
import json_codec
import types

/// Application context passed to handlers
pub type Context {
  Context(
    manager: Subject(ManagerMessage),
    tokens: Dict(String, #(String, String)),  // token -> (user_id, country)
  )
}

/// Main router function
pub fn handle_request(req: Request, ctx: Context) -> Response {
  case wisp.path_segments(req) {
    ["api", "login"] -> handle_login(req, ctx)
    ["api", "rooms"] -> handle_rooms(req, ctx)
    ["api", "rooms", room_id, "info"] -> handle_room_info(req, ctx, room_id)
    ["api", "rooms", room_id, "solve"] -> handle_solve(req, ctx, room_id)
    ["api", "rooms", room_id, "solve_with_note"] -> handle_solve_with_note(req, ctx, room_id)
    ["api", "rooms", room_id, "submit_notes"] -> handle_submit_notes(req, ctx, room_id)
    ["api", "rooms", room_id, "connect"] -> handle_connect_upgrade(req, ctx, room_id)
    ["api", "rooms", room_id, "spectate"] -> handle_spectate_upgrade(req, ctx, room_id)
    _ -> wisp.not_found()
  }
}

/// POST /api/login - Login and get token
fn handle_login(req: Request, ctx: Context) -> Response {
  use <- wisp.require_method(req, Post)
  use body <- wisp.require_json(req)

  case json_codec.decode_login_request(body) {
    Ok(#(username, country)) -> {
      let token = generate_token()
      // Store in tokens map (would need ETS or actor in real impl)
      let response_json = json.object([#("token", json.string(token))])
      wisp.json_response(json.to_string(response_json), 200)
    }
    Error(_) -> wisp.unprocessable_content()
  }
}

/// GET/POST /api/rooms - List or create rooms
fn handle_rooms(req: Request, ctx: Context) -> Response {
  case req.method {
    Get -> {
      let rooms = manager.list_rooms(ctx.manager)
      let response_json = json.array(rooms, json.string)
      wisp.json_response(json.to_string(response_json), 200)
    }
    Post -> {
      // Require auth
      case get_user_from_token(req, ctx.tokens) {
        Some(_user) -> {
          let room_id = manager.generate_room_id()
          case manager.create_room(ctx.manager, room_id) {
            Ok(_) -> {
              let response_json = json.string(room_id)
              wisp.json_response(json.to_string(response_json), 200)
            }
            Error(_) -> wisp.internal_server_error()
          }
        }
        None -> wisp.response(403)
      }
    }
    _ -> wisp.method_not_allowed([Get, Post])
  }
}

/// GET /api/rooms/:id/info - Get room word lists
fn handle_room_info(req: Request, ctx: Context, room_id: String) -> Response {
  use <- wisp.require_method(req, Get)

  case manager.get_room(ctx.manager, room_id) {
    Ok(room) -> {
      let #(allowed_words, banned_words) = process.call(
        room,
        5000,
        fn(reply_to) { room.GetInfo(reply_to) },
      )
      let response_json = json.object([
        #("allowed_words", json.array(allowed_words, json.string)),
        #("banned_words", encode_banned_words(banned_words)),
      ])
      wisp.json_response(json.to_string(response_json), 200)
    }
    Error(_) -> wisp.not_found()
  }
}

/// POST /api/rooms/:id/solve - Check answer
fn handle_solve(req: Request, ctx: Context, room_id: String) -> Response {
  use <- wisp.require_method(req, Post)
  use <- require_auth(req, ctx.tokens)
  use body <- wisp.require_json(req)

  case json_codec.decode_solve_request(body) {
    Ok(answer) -> {
      case manager.get_room(ctx.manager, room_id) {
        Ok(room) -> {
          // Get room info and check answer
          let #(_allowed, banned_words) = process.call(
            room,
            5000,
            fn(reply_to) { room.GetInfo(reply_to) },
          )
          let solved = check_answer(answer, banned_words)
          case solved {
            True -> process.send(room, room.Win)
            False -> Nil
          }
          let response_json = json.object([#("solved", json.bool(solved))])
          wisp.json_response(json.to_string(response_json), 200)
        }
        Error(_) -> wisp.not_found()
      }
    }
    Error(_) -> wisp.unprocessable_content()
  }
}

/// POST /api/rooms/:id/solve_with_note - Check merged notes
fn handle_solve_with_note(req: Request, ctx: Context, room_id: String) -> Response {
  use <- wisp.require_method(req, Post)
  use <- require_auth(req, ctx.tokens)

  case manager.get_room(ctx.manager, room_id) {
    Ok(_room) -> {
      // TODO: Get all player notes and check
      let response_json = json.object([#("solved", json.bool(False))])
      wisp.json_response(json.to_string(response_json), 200)
    }
    Error(_) -> wisp.not_found()
  }
}

/// POST /api/rooms/:id/submit_notes - Submit player's guesses
fn handle_submit_notes(req: Request, ctx: Context, room_id: String) -> Response {
  use <- wisp.require_method(req, Post)
  use body <- wisp.require_json(req)

  case get_user_from_token(req, ctx.tokens) {
    Some(#(user_id, country)) -> {
      case json_codec.decode_submit_notes_request(body) {
        Ok(notes) -> {
          case manager.get_room(ctx.manager, room_id) {
            Ok(room) -> {
              // Send submit notes action
              process.send(room, room.ProcessAction(
                user_id,
                country,
                types.SubmitNotes(notes)
              ))

              // Check victory
              let victory = process.call(
                room,
                5000,
                fn(reply_to) { room.CheckVictory(reply_to) },
              )

              let response_json = json.object([
                #("success", json.bool(True)),
                #("discovered_count", json.int(0)),
                #("total_required", json.int(0)),
                #("victory_achieved", json.bool(victory)),
              ])
              wisp.json_response(json.to_string(response_json), 200)
            }
            Error(_) -> wisp.not_found()
          }
        }
        Error(_) -> wisp.unprocessable_content()
      }
    }
    None -> wisp.response(403)
  }
}

/// Placeholder for WebSocket upgrade (handled by mist directly)
fn handle_connect_upgrade(_req: Request, _ctx: Context, _room_id: String) -> Response {
  wisp.response(101)  // Upgrade handled elsewhere
}

fn handle_spectate_upgrade(_req: Request, _ctx: Context, _room_id: String) -> Response {
  wisp.response(101)  // Upgrade handled elsewhere
}

// Helper functions

fn get_user_from_token(req: Request, tokens: Dict(String, #(String, String))) -> Option(#(String, String)) {
  case list.find(req.headers, fn(h) { h.0 == "x-user-token" }) {
    Ok(#(_, token)) -> dict.get(tokens, token) |> option.from_result
    Error(_) -> None
  }
}

fn require_auth(req: Request, tokens: Dict(String, #(String, String)), next: fn() -> Response) -> Response {
  case get_user_from_token(req, tokens) {
    Some(_) -> next()
    None -> wisp.response(403)
  }
}

fn check_answer(answer: Dict(String, List(String)), banned_words: Dict(String, List(String))) -> Bool {
  case dict.size(answer) == dict.size(banned_words) {
    False -> False
    True -> {
      dict.to_list(answer)
      |> list.all(fn(pair) {
        let #(country, submitted) = pair
        case dict.get(banned_words, country) {
          Ok(expected) -> {
            let s1 = set.from_list(submitted)
            let s2 = set.from_list(expected)
            s1 == s2
          }
          Error(_) -> False
        }
      })
    }
  }
}

fn encode_banned_words(banned: Dict(String, List(String))) -> json.Json {
  let entries = dict.to_list(banned)
  json.object(
    list.map(entries, fn(entry) {
      let #(country, words) = entry
      #(country, json.array(words, json.string))
    })
  )
}

fn generate_token() -> String {
  manager.generate_room_id()  // Reuse the random string generator
}
