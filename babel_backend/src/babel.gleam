import gleam/bytes_tree
import gleam/dict
import gleam/erlang/process
import gleam/http/request.{type Request}
import gleam/http/response.{type Response}
import gleam/io
import gleam/option.{None, Some}
import gleam/result
import gleam/string
import mist.{type Connection, type ResponseData}
import wisp
import actors/manager
import web/router
import web/socket

/// Application state
pub type AppState {
  AppState(
    manager: process.Subject(manager.ManagerMessage),
    tokens: dict.Dict(String, #(String, String)),
  )
}

pub fn main() {
  // Configure logging
  wisp.configure_logger()

  io.println("Starting Project Babel backend...")

  // Start the room manager actor
  let assert Ok(room_manager) = manager.start()
  io.println("Room manager started")

  // Create initial app state with empty tokens (in prod, use ETS)
  let tokens = dict.new()

  // Build the request handler
  let handler = fn(req: Request(Connection)) -> Response(ResponseData) {
    handle_request(req, room_manager, tokens)
  }

  // Start mist web server
  let assert Ok(_) =
    mist.new(handler)
    |> mist.port(3000)
    |> mist.start

  io.println("Server listening on http://0.0.0.0:3000")

  // Keep the main process running
  process.sleep_forever()
}

fn handle_request(
  req: Request(Connection),
  mgr: process.Subject(manager.ManagerMessage),
  tokens: dict.Dict(String, #(String, String)),
) -> Response(ResponseData) {
  let path_segments = string.split(req.path, "/")

  // Handle WebSocket upgrades separately
  case path_segments {
    ["", "api", "rooms", _room_id, "connect"] -> {
      socket.handle_participant_websocket(req, mgr, tokens)
    }
    ["", "api", "rooms", _room_id, "spectate"] -> {
      socket.handle_spectator_websocket(req, mgr)
    }
    _ -> {
      // Regular HTTP requests - convert request and route
      let ctx = router.Context(manager: mgr, tokens: tokens)

      // Convert mist request to wisp request
      let wisp_req = request.map(req, fn(_) { wisp.Empty })

      // Handle the request
      let wisp_response = router.handle_request(wisp_req, ctx)

      // Convert wisp response back to mist
      wisp_response_to_mist(wisp_response)
    }
  }
}

fn wisp_response_to_mist(resp: wisp.Response) -> Response(ResponseData) {
  case resp.body {
    wisp.Empty -> {
      response.new(resp.status)
      |> response.set_body(mist.Bytes(bytes_tree.new()))
    }
    wisp.Text(text) -> {
      response.new(resp.status)
      |> response.set_body(mist.Bytes(bytes_tree.from_string_tree(text)))
    }
    wisp.Bytes(data) -> {
      response.new(resp.status)
      |> response.set_body(mist.Bytes(data))
    }
    wisp.File(_path) -> {
      // Simplified - just return empty for now
      response.new(resp.status)
      |> response.set_body(mist.Bytes(bytes_tree.new()))
    }
  }
}
