use babel::data::*;
use babel::manager::RoomManager;
use babel::room::ChatRoom;
use babel::server::{AppState, build_router};
use babel::utils::deserialize_from_file;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::net::TcpListener;

static FILTER_CONFIG: Lazy<FilterConfig> =
    Lazy::new(|| deserialize_from_file("filter_config.json"));

pub struct DefaultRoomConfig;

impl RoomConfig for DefaultRoomConfig {
    fn get_filter_config(&self) -> &'static FilterConfig {
        &FILTER_CONFIG
    }

    fn init_room(&self, room_id: RoomId) -> ChatRoom {
        ChatRoom::new(room_id, &FILTER_CONFIG)
    }
}

#[tokio::main]
async fn main() {
    let room_manager = RoomManager::from_config(DefaultRoomConfig);

    // Load user tokens: maps token -> (user_id, country)
    let tokens_map: HashMap<String, (UserId, CountryCode)> =
        deserialize_from_file("user_tokens.json");

    let state = AppState {
        room_manager,
        tokens_map,
    };

    let app = build_router(state);

    let addr = "0.0.0.0:3000";
    eprintln!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
