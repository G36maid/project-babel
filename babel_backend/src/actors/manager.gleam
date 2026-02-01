import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/list
import gleam/otp/actor
import gleam/string
import actors/room.{type RoomMessage}

/// State for the room manager
pub type ManagerState {
  ManagerState(
    rooms: Dict(String, Subject(RoomMessage)),
  )
}

/// Messages the manager can receive
pub type ManagerMessage {
  CreateRoom(room_id: String, reply_to: Subject(Result(Subject(RoomMessage), String)))
  GetRoom(room_id: String, reply_to: Subject(Result(Subject(RoomMessage), String)))
  GetOrCreateRoom(room_id: String, reply_to: Subject(Subject(RoomMessage)))
  ListRooms(reply_to: Subject(List(String)))
  RemoveRoom(room_id: String)
}

/// Start the room manager actor
pub fn start() -> Result(Subject(ManagerMessage), actor.StartError) {
  let state = ManagerState(rooms: dict.new())
  let builder = actor.new(state)
  |> actor.on_message(fn(state, msg) { handle_message(msg, state) })
  case actor.start(builder) {
    Ok(started) -> Ok(started.data)
    Error(e) -> Error(e)
  }
}

fn handle_message(msg: ManagerMessage, state: ManagerState) -> actor.Next(ManagerState, ManagerMessage) {
  case msg {
    CreateRoom(room_id, reply_to) -> {
      case dict.get(state.rooms, room_id) {
        Ok(existing) -> {
          process.send(reply_to, Ok(existing))
          actor.continue(state)
        }
        Error(_) -> {
          case room.start(room_id) {
            Ok(room_subject) -> {
              let new_rooms = dict.insert(state.rooms, room_id, room_subject)
              process.send(reply_to, Ok(room_subject))
              actor.continue(ManagerState(rooms: new_rooms))
            }
            Error(_) -> {
              process.send(reply_to, Error("Failed to start room"))
              actor.continue(state)
            }
          }
        }
      }
    }

    GetRoom(room_id, reply_to) -> {
      case dict.get(state.rooms, room_id) {
        Ok(room_subject) -> {
          process.send(reply_to, Ok(room_subject))
        }
        Error(_) -> {
          process.send(reply_to, Error("Room not found"))
        }
      }
      actor.continue(state)
    }

    GetOrCreateRoom(room_id, reply_to) -> {
      case dict.get(state.rooms, room_id) {
        Ok(room_subject) -> {
          process.send(reply_to, room_subject)
          actor.continue(state)
        }
        Error(_) -> {
          case room.start(room_id) {
            Ok(room_subject) -> {
              let new_rooms = dict.insert(state.rooms, room_id, room_subject)
              process.send(reply_to, room_subject)
              actor.continue(ManagerState(rooms: new_rooms))
            }
            Error(_) -> {
              // This shouldn't happen, but handle gracefully
              actor.continue(state)
            }
          }
        }
      }
    }

    ListRooms(reply_to) -> {
      let room_ids = dict.keys(state.rooms)
      process.send(reply_to, room_ids)
      actor.continue(state)
    }

    RemoveRoom(room_id) -> {
      let new_rooms = dict.delete(state.rooms, room_id)
      actor.continue(ManagerState(rooms: new_rooms))
    }
  }
}

/// Helper function to create a room via the manager
pub fn create_room(manager: Subject(ManagerMessage), room_id: String) -> Result(Subject(RoomMessage), String) {
  process.call(manager, 5000, fn(reply_to) { CreateRoom(room_id, reply_to) })
}

/// Helper function to get a room
pub fn get_room(manager: Subject(ManagerMessage), room_id: String) -> Result(Subject(RoomMessage), String) {
  process.call(manager, 5000, fn(reply_to) { GetRoom(room_id, reply_to) })
}

/// Helper function to get or create a room
pub fn get_or_create_room(manager: Subject(ManagerMessage), room_id: String) -> Subject(RoomMessage) {
  process.call(manager, 5000, fn(reply_to) { GetOrCreateRoom(room_id, reply_to) })
}

/// Helper function to list all rooms
pub fn list_rooms(manager: Subject(ManagerMessage)) -> List(String) {
  process.call(manager, 5000, ListRooms)
}

/// Helper to generate a random room ID (16 chars alphanumeric)
pub fn generate_room_id() -> String {
  random_string(16)
}

@external(erlang, "crypto", "strong_rand_bytes")
fn crypto_strong_rand_bytes(n: Int) -> BitArray

fn random_string(length: Int) -> String {
  let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
  let chars_len = string.length(chars)
  let bytes = crypto_strong_rand_bytes(length)

  bytes
  |> bit_array_to_list
  |> list.map(fn(byte) {
    let idx = byte % chars_len
    string.slice(chars, idx, 1)
  })
  |> string.concat
}

@external(erlang, "erlang", "binary_to_list")
fn bit_array_to_list(bits: BitArray) -> List(Int)
