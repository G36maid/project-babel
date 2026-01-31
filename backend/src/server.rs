use serde::Serialize;
#[derive(Serialize)]
struct RoomWordsInfo {
    allowed_words: Vec<String>,
    banned_words: std::collections::HashMap<String, Vec<String>>,
}

// GET /api/rooms/{id}/info - Return allowed and banned words for the room
async fn get_room_words_info(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
) -> Result<Json<RoomWordsInfo>, StatusCode> {
    let connector = state.room_manager.connect_to_room(&room_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let allowed_words = (*connector.allowed_words).clone();
    let banned_words = (*connector.banned_words).clone();
    Ok(Json(RoomWordsInfo { allowed_words, banned_words }))
}
use axum::{
    Json as AxumJson, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{Json, Response},
    routing::{get, post}
};
use rand::distr::{Alphanumeric, SampleString};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::wrappers::WatchStream;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, info, warn};

use crate::data::*;
use crate::manager::{RoomConnector, RoomManager};

#[derive(Clone)]
pub struct AppState {
    pub room_manager: Arc<RoomManager>,
    pub tokens_map: HashMap<String, (UserId, CountryCode)>,
}

#[derive(Deserialize)]
pub struct ConnectQuery {
    token: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    country: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    token: String,
}
// POST /api/login - Login and get token
async fn login(
    State(mut state): State<AppState>,
    AxumJson(payload): AxumJson<LoginRequest>,
) -> Result<AxumJson<LoginResponse>, StatusCode> {
    // Generate a random token
    let token: String = Alphanumeric.sample_string(&mut rand::rng(), 16);

    // Insert into tokens_map
    state
        .tokens_map
        .insert(token.clone(), (payload.username, payload.country));

    Ok(AxumJson(LoginResponse { token }))
}

pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub country: CountryCode,
}

fn extract_user_from_headers(
    headers: &HeaderMap,
    tokens_map: &HashMap<String, (UserId, CountryCode)>,
) -> Option<AuthenticatedUser> {
    let token = headers.get("X-User-Token")?.to_str().ok()?;
    let (user_id, country) = tokens_map.get(token)?;
    Some(AuthenticatedUser {
        user_id: user_id.clone(),
        country: country.clone(),
    })
}

// GET /api/info - Filter config
async fn get_info(State(state): State<AppState>) -> Json<RoomInfo> {
    let config = state.room_manager.get_filter_config();
    Json(RoomInfo {
        filter_config: config.clone(),
    })
}

// GET /api/rooms - List room IDs
async fn list_rooms(State(state): State<AppState>) -> Json<Vec<RoomId>> {
    Json(state.room_manager.list_rooms())
}

// POST /api/rooms - Create room (requires auth)
async fn create_room(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RoomId>, StatusCode> {
    let _user = extract_user_from_headers(&headers, &state.tokens_map).ok_or_else(|| {
        warn!("Unauthorized room creation attempt");
        StatusCode::FORBIDDEN
    })?;

    let room_id = Arc::clone(&state.room_manager).create_and_run_room();
    info!(room_id, "Room created");
    Ok(Json(room_id))
}

// GET /api/rooms/:id/connect - WebSocket for participants
async fn connect_room(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
    Query(query): Query<ConnectQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    let user = extract_user_from_headers(&headers, &state.tokens_map).ok_or_else(|| {
        warn!(room_id, "Unauthorized connection attempt");
        StatusCode::FORBIDDEN
    })?;

    eprintln!(
        "User {} ({}) connecting to room {}",
        user.user_id, user.country, room_id
    );

    // Get or create room if it doesn't exist (especially for test_room)
    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .unwrap_or_else(|| {
            warn!(room_id, "Connection attempt to non-existent room");
            let new_room_id = Arc::clone(&state.room_manager).create_room_with_id(room_id.clone());
            state.room_manager.connect_to_room(&new_room_id).unwrap()
        });

    let user_id = user.user_id.clone();
    info!(room_id, user_id, "User connecting to room");
    Ok(ws.on_upgrade(move |socket| handle_participant_socket(socket, connector, user, room_id)))
}

async fn handle_participant_socket(
    socket: WebSocket,
    connector: RoomConnector,
    user: AuthenticatedUser,
    room_id: RoomId,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let RoomConnector {
        action_sender,
        mut update_receiver,
        allowed_words: _,
        banned_words: _,
    } = connector;

    let user_id = &user.user_id;
    debug!(room_id, user_id, "WebSocket connection established");

    // Send initial join action
    let join_message = UserMessage {
        user_id: user.user_id.clone(),
        country: user.country.clone(),
        action: UserAction::SendMessage(String::new()), // Empty message to trigger join
    };
    if action_sender.send(join_message).await.is_err() {
        warn!(room_id, user_id, "Failed to send join message");
        return;
    }

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        eprintln!("Received message from {}: {}", user.user_id, text);
                        if let Ok(action) = from_str::<UserAction>(&text) {
                            debug!(room_id, user_id, ?action, "Received action");
                            let user_message = UserMessage {
                                user_id: user.user_id.clone(),
                                country: user.country.clone(),
                                action,
                            };
                            if action_sender.send(user_message).await.is_err() {
                                warn!(room_id, user_id, "Failed to send action");
                                break;
                            }
                        } else {
                            eprintln!("Failed to parse action from: {}", text);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!(room_id, user_id, "User disconnecting");
                        // Send leave action
                        let leave_message = UserMessage {
                            user_id: user.user_id.clone(),
                            country: user.country.clone(),
                            action: UserAction::LeaveRoom,
                        };
                        let _ = action_sender.send(leave_message).await;
                        break;
                    }
                    _ => {}
                }
            }
            result = update_receiver.changed() => {
                match result {
                    Ok(_) => {
                        let update = update_receiver.borrow().clone();
                        eprintln!("Sending update to {}: {} new messages, {} notifications",
                            user.user_id, update.new_messages.len(), update.notifications.len());
                        if let Ok(json) = to_string(&update) {
                            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                                debug!(room_id, user_id, "Failed to send update, closing connection");
                                break;
                            }
                        }
                        if update.room_closed {
                            info!(room_id, user_id, "Room closed");
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

// GET /api/rooms/:id/spectate - WebSocket for spectators (read-only)
async fn spectate_room(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .ok_or_else(|| {
            warn!(room_id, "Spectate attempt on non-existent room");
            StatusCode::NOT_FOUND
        })?;

    info!(room_id, "Spectator connecting");
    Ok(ws.on_upgrade(move |socket| handle_spectator_socket(socket, connector, room_id)))
}

async fn handle_spectator_socket(socket: WebSocket, connector: RoomConnector, room_id: RoomId) {
    let (mut ws_sender, _ws_receiver) = socket.split();
    let mut update_stream = WatchStream::new(connector.update_receiver);

    debug!(room_id, "Spectator WebSocket established");

    while let Some(update) = update_stream.next().await {
        if let Ok(json) = to_string(&update) {
            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                debug!(room_id, "Spectator disconnected");
                break;
            }
        }
        if update.room_closed {
            info!(room_id, "Room closed, spectator disconnecting");
            break;
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/info", get(get_info))
        .route("/api/rooms", get(list_rooms))
        .route("/api/rooms", post(create_room))
        .route("/api/rooms/{id}/connect", get(connect_room))
        .route("/api/rooms/{id}/spectate", get(spectate_room))
        .route("/api/rooms/{id}/info", get(get_room_words_info))
        .route("/api/login", post(login))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
