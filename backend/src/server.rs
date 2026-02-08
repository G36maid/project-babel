// POST /api/rooms/{id}/solve_with_note - Use merged player notes to solve
async fn solve_room_with_note(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
    headers: HeaderMap,
) -> Result<Json<SolveResponse>, StatusCode> {
    let _user =
        extract_user_from_headers(&headers, &state.tokens_map).ok_or(StatusCode::FORBIDDEN)?;
    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let mut room = connector.room.lock().unwrap();
    // 所有玩家的 player note 都要答對才算勝利
    let notes: Vec<_> = room.player_notes.values().cloned().collect();
    let mut all_correct = true;
    for note in notes {
        if !solve_answer(&mut room, note) {
            all_correct = false;
            break;
        }
    }
    if all_correct {
        room.win();
    }
    Ok(Json(SolveResponse {
        solved: all_correct,
    }))
}
// Helper for answer checking and success action, used by both solve_room and solve_room_with_note
fn solve_answer(room: &mut crate::room::ChatRoom, answer: HashMap<String, Vec<String>>) -> bool {
    let banned_words = &room.filter.config.banned_words;
    if answer.len() != banned_words.len() {
        return false;
    }
    for (country, submitted) in &answer {
        match banned_words.get(country) {
            Some(expected) => {
                let s1: std::collections::HashSet<_> = submitted.iter().collect();
                let s2: std::collections::HashSet<_> = expected.iter().collect();
                if s1 != s2 {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}
use std::collections::HashMap;

use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Deserialize, Serialize, ToSchema)]
struct SolveResponse {
    solved: bool,
}
// POST /api/rooms/{id}/solve - Check if submitted banned words match exactly
#[derive(Deserialize, ToSchema)]
struct SolveRequest {
    answer: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct SubmitNotesRequest {
    notes: HashMap<String, Vec<String>>,
}

#[derive(Serialize)]
struct SubmitNotesResponse {
    success: bool,
    discovered_count: usize,
    total_required: usize,
    victory_achieved: bool,
}

// POST /api/rooms/{id}/solve - Check if submitted banned words match exactly, and send system message if correct
#[utoipa::path(
    post,
    path = "/api/rooms/{id}/solve",
    params(
        ("id" = String, Path, description = "Room ID")
    ),
    request_body = SolveRequest,
    responses(
        (status = 200, description = "Solution check result", body = SolveResponse),
        (status = 403, description = "Forbidden (invalid token)"),
        (status = 404, description = "Room not found")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn solve_room(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
    headers: HeaderMap, // 新增
    AxumJson(payload): AxumJson<SolveRequest>,
) -> Result<Json<SolveResponse>, StatusCode> {
    let _user =
        extract_user_from_headers(&headers, &state.tokens_map).ok_or(StatusCode::FORBIDDEN)?;
    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let mut room = connector.room.lock().unwrap();
    let result = solve_answer(&mut room, payload.answer);
    if result {
        room.win();
    }
    Ok(Json(SolveResponse { solved: result }))
}

// POST /api/rooms/{id}/submit_notes - Submit player's guesses for banned words
async fn submit_notes(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
    headers: HeaderMap,
    AxumJson(payload): AxumJson<SubmitNotesRequest>,
) -> Result<Json<SubmitNotesResponse>, StatusCode> {
    let user =
        extract_user_from_headers(&headers, &state.tokens_map).ok_or(StatusCode::FORBIDDEN)?;

    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut room = connector.room.lock().unwrap();

    // Update player's notes
    room.player_notes
        .insert(user.user_id.clone(), payload.notes);

    eprintln!("[SubmitNotes] User {} submitted notes", user.user_id);

    // Calculate progress for this specific user
    let all_progress = room.get_player_progress();
    let user_progress = all_progress.iter().find(|p| p.user_id == user.user_id);

    let discovered_count = user_progress.map(|p| p.discovered_count).unwrap_or(0);
    let total_required = room
        .filter
        .config
        .banned_words
        .values()
        .map(|words| words.len())
        .sum();

    eprintln!(
        "[SubmitNotes] Progress: {} discovered {} / {} words",
        user.user_id, discovered_count, total_required
    );

    // Check victory
    let victory_achieved = room.check_victory();

    eprintln!("[SubmitNotes] Victory check result: {}", victory_achieved);
    eprintln!("[SubmitNotes] All progress: {:?}", all_progress);

    // If victory achieved, broadcast update
    if victory_achieved {
        eprintln!("[SubmitNotes] Broadcasting victory state!");
        // Create and send victory update
        let victory_state = room.get_victory_state();
        let room_state = room.get_censored_state_for(&"".to_string());

        drop(room); // Release lock before sending

        let update = RoomUpdate {
            room_state,
            new_messages: vec![],
            notifications: vec![Notification {
                message: "🎉 Victory! All players discovered all banned words!".to_string(),
            }],
            room_closed: false,
            victory: Some(victory_state),
        };

        connector.update_sender.send_replace(update);
    } else {
        drop(room); // Release lock
    }

    Ok(Json(SubmitNotesResponse {
        success: true,
        discovered_count,
        total_required,
        victory_achieved,
    }))
}

use futures::SinkExt;
use futures::StreamExt;
use serde::Serialize;
#[derive(Serialize, ToSchema)]
struct RoomWordsInfo {
    allowed_words: Vec<String>,
    banned_words: std::collections::HashMap<String, Vec<String>>,
}

// GET /api/rooms/{id}/info - Return allowed and banned words for the room
#[utoipa::path(
    get,
    path = "/api/rooms/{id}/info",
    params(
        ("id" = String, Path, description = "Room ID")
    ),
    responses(
        (status = 200, description = "Room words info", body = RoomWordsInfo),
        (status = 404, description = "Room not found")
    )
)]
async fn get_room_words_info(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
) -> Result<Json<RoomWordsInfo>, StatusCode> {
    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let room = connector.room.lock().unwrap();
    let allowed_words = room.allowed_words.clone();
    let banned_words = room.filter.config.banned_words.clone();
    Ok(Json(RoomWordsInfo {
        allowed_words,
        banned_words,
    }))
}
use axum::{
    Json as AxumJson, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message as WsMessage, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{Json, Response},
    routing::{get, post},
};
use dashmap::DashMap;
use rand::distr::{Alphanumeric, SampleString};
use serde::Deserialize;
use serde_json::{from_str, to_string};
use std::sync::Arc;
use tokio_stream::wrappers::WatchStream;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{debug, info, warn};

use crate::data::*;
use crate::manager::{RoomConnector, RoomManager};

/// Update sent to clients with messages censored for their specific country.
#[derive(Clone, Debug, Serialize, ToSchema)]
struct ClientRoomUpdate {
    room_state: RoomState,
    new_messages: Vec<CensoredMessage>,
    notifications: Vec<Notification>,
    room_closed: bool,
    victory: Option<VictoryState>,
}

#[derive(Clone)]
pub struct AppState {
    pub room_manager: Arc<RoomManager>,
    pub tokens_map: Arc<DashMap<String, (UserId, CountryCode)>>,
}

#[derive(Deserialize, ToSchema)]
pub struct ConnectQuery {
    token: String,
}

#[derive(Deserialize, ToSchema)]
struct LoginRequest {
    username: String,
    country: String,
}

#[derive(serde::Serialize, ToSchema)]
struct LoginResponse {
    token: String,
}
// POST /api/login - Login and get token
#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse)
    )
)]
async fn login(
    State(state): State<AppState>,
    AxumJson(payload): AxumJson<LoginRequest>,
) -> Result<AxumJson<LoginResponse>, StatusCode> {
    // Generate a random token
    let token: String = Alphanumeric.sample_string(&mut rand::rng(), 16);

    // Insert into tokens_map (DashMap)
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
    tokens_map: &DashMap<String, (UserId, CountryCode)>,
) -> Option<AuthenticatedUser> {
    let token = headers.get("X-User-Token")?.to_str().ok()?;
    let pair = tokens_map.get(token)?;
    let (user_id, country) = pair.value();
    Some(AuthenticatedUser {
        user_id: user_id.clone(),
        country: country.clone(),
    })
}

// GET /api/info - Filter config
#[utoipa::path(
    get,
    path = "/api/info",
    responses(
        (status = 200, description = "Get global filter config", body = RoomInfo)
    )
)]
async fn get_info(State(state): State<AppState>) -> Json<RoomInfo> {
    let config = state.room_manager.get_filter_config();
    Json(RoomInfo {
        filter_config: config.clone(),
    })
}

// GET /api/rooms - List room IDs
#[utoipa::path(
    get,
    path = "/api/rooms",
    responses(
        (status = 200, description = "List active room IDs", body = Vec<RoomId>)
    )
)]
async fn list_rooms(State(state): State<AppState>) -> Json<Vec<RoomId>> {
    Json(state.room_manager.list_rooms())
}

// POST /api/rooms - Create room (requires auth)
#[utoipa::path(
    post,
    path = "/api/rooms",
    responses(
        (status = 200, description = "Room created", body = RoomId),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("api_key" = [])
    )
)]
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
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    // Extract user from query parameter token
    let (user_id, country) = state
        .tokens_map
        .get(&query.token)
        .map(|pair| pair.value().clone())
        .ok_or_else(|| {
            warn!(room_id, token = %query.token, "Unauthorized connection attempt");
            StatusCode::FORBIDDEN
        })?;

    let user = AuthenticatedUser {
        user_id: user_id.clone(),
        country: country.clone(),
    };

    info!(room_id, user_id = %user.user_id, country = %user.country, "User connecting to room");

    // Get or create room if it doesn't exist (especially for test_room)
    let connector = state
        .room_manager
        .connect_to_room(&room_id)
        .unwrap_or_else(|| {
            info!(room_id, "Room not found, creating it");
            let new_room_id = Arc::clone(&state.room_manager).create_room_with_id(room_id.clone());
            state.room_manager.connect_to_room(&new_room_id).unwrap()
        });

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
        update_sender: _,
        room,
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
                    Some(Ok(WsMessage::Text(text))) => {
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
                    Some(Ok(WsMessage::Close(_))) | None => {
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

                        // Lock the room only to extract the needed fields, then drop the lock before await
                        // Censor messages for this user's country using the room's method
                        let censored_messages: Vec<CensoredMessage> = {
                            let locked_room = room.lock().unwrap();
                            update
                                .new_messages
                                .iter()
                                .map(|msg| locked_room.censor_message_for(msg, &user.country))
                                .collect()
                        };

                        let client_update = ClientRoomUpdate {
                            room_state: update.room_state,
                            new_messages: censored_messages,
                            notifications: update.notifications,
                            room_closed: update.room_closed,
                            victory: update.victory,
                        };

                        debug!(
                            room_id,
                            user_id,
                            user_country = %user.country,
                            message_count = client_update.new_messages.len(),
                            "Sending censored update to client"
                        );

                        #[allow(clippy::collapsible_if)]
                        if let Ok(json) = to_string(&client_update) {
                            if ws_sender.send(WsMessage::Text(json.into())).await.is_err() {
                                debug!(room_id, user_id, "Failed to send update, closing connection");
                                break;
                            }
                        }
                        if client_update.room_closed {
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
        #[allow(clippy::collapsible_if)]
        if let Ok(json) = to_string(&update) {
            if ws_sender.send(WsMessage::Text(json.into())).await.is_err() {
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

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        get_info,
        list_rooms,
        create_room,
        get_room_words_info,
        solve_room
    ),
    components(
        schemas(
            LoginRequest,
            LoginResponse,
            RoomInfo,
            RoomId,
            RoomWordsInfo,
            SolveRequest,
            SolveResponse,
            ClientRoomUpdate,
            ConnectQuery,
            crate::data::FilterConfig,
            crate::data::Message,
            crate::data::CensoredMessage,
            crate::data::UserAction,
            crate::data::Participant,
            crate::data::RoomState,
            crate::data::Notification,
            crate::data::RoomUpdate
        )
    ),
    tags(
        (name = "babel", description = "Project Babel API")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-User-Token"),
                    ),
                ),
            )
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/info", get(get_info))
        .route("/api/rooms", get(list_rooms))
        .route("/api/rooms", post(create_room))
        .route("/api/rooms/{id}/connect", get(connect_room))
        .route("/api/rooms/{id}/spectate", get(spectate_room))
        .route("/api/rooms/{id}/info", get(get_room_words_info))
        .route("/api/rooms/{id}/solve", post(solve_room))
        .route(
            "/api/rooms/{id}/solve_with_note",
            post(solve_room_with_note),
        )
        .route("/api/rooms/{id}/submit_notes", post(submit_notes))
        .route("/api/login", post(login))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    // Note: Censorship tests are located in room.rs
    // This module is kept for potential future server-specific tests
}
