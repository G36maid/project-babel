import gleam/bytes_tree
import gleam/dict.{type Dict}
import gleam/dynamic/decode
import gleam/erlang/process.{type Subject}
import gleam/http/request.{type Request}
import gleam/http/response.{type Response}
import gleam/json
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam.{type Nil}
import gleam/string
import mist.{type Connection, type ResponseData, type WebsocketConnection}
import actors/manager.{type ManagerMessage}
import actors/room.{type RoomMessage}
import json_codec
import types

/// WebSocket state for a participant
pub type ParticipantState {
  ParticipantState(
    user_id: String,
    country: String,
    room: Subject(RoomMessage),
    update_subject: Subject(types.RoomUpdate),
  )
}

/// WebSocket state for a spectator
pub type SpectatorState {
  SpectatorState(
    room: Subject(RoomMessage),
    update_subject: Subject(types.RoomUpdate),
  )
}

/// Handle participant WebSocket connection
pub fn handle_participant_websocket(
  req: Request(Connection),
  manager: Subject(ManagerMessage),
  tokens: Dict(String, #(String, String)),
) -> Response(ResponseData) {
  // Extract token from query string
  let token = get_query_param(req, "token")
  let room_id = get_room_id_from_path(req)

  case token, room_id {
    Some(t), Some(rid) -> {
      case dict.get(tokens, t) {
        Ok(#(user_id, country)) -> {
          let room = manager.get_or_create_room(manager, rid)

          // Create update receiver subject
          let update_subject = process.new_subject()

          // Subscribe to room updates
          process.send(room, room.Subscribe(update_subject))

          // Send join
          let _join_result = process.call(
            room,
            5000,
            fn(reply_to) { room.Join(user_id, country, reply_to) },
          )

          let state = ParticipantState(
            user_id: user_id,
            country: country,
            room: room,
            update_subject: update_subject,
          )

          mist.websocket(
            request: req,
            on_init: fn(_conn) { #(state, None) },
            on_close: fn(state: ParticipantState) {
              process.send(state.room, room.Leave(state.user_id))
              process.send(state.room, room.Unsubscribe(state.update_subject))
            },
            handler: handle_participant_message,
          )
        }
        Error(_) -> {
          response.new(403)
          |> response.set_body(mist.Bytes(bytes_tree.new()))
        }
      }
    }
    _, _ -> {
      response.new(400)
      |> response.set_body(mist.Bytes(bytes_tree.new()))
    }
  }
}

fn handle_participant_message(
  state: ParticipantState,
  message: mist.WebsocketMessage(types.RoomUpdate),
  _conn: WebsocketConnection,
) -> mist.Next(ParticipantState, types.RoomUpdate) {
  case message {
    mist.Text(text) -> {
      // Parse user action from JSON
      let decoder = json_codec.make_user_action_decoder()
      case json.parse(text, decoder) {
        Ok(action) -> {
          process.send(state.room, room.ProcessAction(
            state.user_id,
            state.country,
            action
          ))
        }
        Error(_) -> Nil
      }
      mist.continue(state)
    }

    mist.Binary(_) -> mist.continue(state)

    mist.Custom(_update) -> {
      // Handle room update - would send to client
      mist.continue(state)
    }

    mist.Closed | mist.Shutdown -> mist.stop()
  }
}

/// Handle spectator WebSocket connection
pub fn handle_spectator_websocket(
  req: Request(Connection),
  manager: Subject(ManagerMessage),
) -> Response(ResponseData) {
  let room_id = get_room_id_from_path(req)

  case room_id {
    Some(rid) -> {
      case manager.get_room(manager, rid) {
        Ok(room) -> {
          let update_subject = process.new_subject()
          process.send(room, room.Subscribe(update_subject))

          let state = SpectatorState(
            room: room,
            update_subject: update_subject,
          )

          mist.websocket(
            request: req,
            on_init: fn(_conn) { #(state, None) },
            on_close: fn(state: SpectatorState) {
              process.send(state.room, room.Unsubscribe(state.update_subject))
            },
            handler: handle_spectator_message,
          )
        }
        Error(_) -> {
          response.new(404)
          |> response.set_body(mist.Bytes(bytes_tree.new()))
        }
      }
    }
    None -> {
      response.new(400)
      |> response.set_body(mist.Bytes(bytes_tree.new()))
    }
  }
}

fn handle_spectator_message(
  state: SpectatorState,
  message: mist.WebsocketMessage(types.RoomUpdate),
  _conn: WebsocketConnection,
) -> mist.Next(SpectatorState, types.RoomUpdate) {
  case message {
    mist.Text(_) -> mist.continue(state)  // Spectators can't send actions
    mist.Binary(_) -> mist.continue(state)
    mist.Custom(_) -> mist.continue(state)
    mist.Closed | mist.Shutdown -> mist.stop()
  }
}

// Helper functions

fn get_query_param(req: Request(Connection), key: String) -> Option(String) {
  case request.get_query(req) {
    Ok(params) -> {
      list.find(params, fn(p) { p.0 == key })
      |> result.map(fn(p) { p.1 })
      |> option.from_result
    }
    Error(_) -> None
  }
}

fn get_room_id_from_path(req: Request(Connection)) -> Option(String) {
  let segments = string.split(req.path, "/")
  // Path: /api/rooms/{id}/connect or /api/rooms/{id}/spectate
  case segments {
    ["", "api", "rooms", room_id, _] -> Some(room_id)
    _ -> None
  }
}
