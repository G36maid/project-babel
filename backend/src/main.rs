use babel::data::*;
use babel::manager::RoomManager;
use babel::room::ChatRoom;
use babel::server::{AppState, build_router};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static FILTER_CONFIG: Lazy<FilterConfig> = Lazy::new(FilterConfig::default);

pub struct DefaultRoomConfig;

impl RoomConfig for DefaultRoomConfig {
    fn get_filter_config(&self) -> &FilterConfig {
        &FILTER_CONFIG
    }

    fn init_room(&self, room_id: RoomId) -> Box<dyn Room> {
        Box::new(ChatRoom::new(room_id, &FILTER_CONFIG))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "babel=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initializing server");

    let room_manager = RoomManager::from_config(DefaultRoomConfig);

    let state = AppState {
        room_manager,
        tokens_map: Arc::new(DashMap::new()),
    };

    let app = build_router(state);

    let addr = "0.0.0.0:3000";
    info!(addr, "Starting server");

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
