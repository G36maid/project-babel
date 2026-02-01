use dashmap::DashMap;
use rand::distr::{Alphanumeric, SampleString};
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

use crate::data::*;
use crate::room::ChatRoom;

pub struct RoomRunner {
    room: ChatRoom,
    action_receiver: mpsc::Receiver<UserMessage>,
    update_sender: watch::Sender<RoomUpdate>,
    room_manager: Arc<RoomManager>,
}

#[derive(Clone)]
pub struct RoomConnector {
    pub action_sender: mpsc::Sender<UserMessage>,
    pub update_receiver: watch::Receiver<RoomUpdate>,
    pub allowed_words: Arc<Vec<String>>,
    pub banned_words: Arc<std::collections::HashMap<String, Vec<String>>>,
}

pub struct RoomManager {
    active_rooms: DashMap<RoomId, RoomConnector>,
    config: Box<dyn RoomConfig>,
}

impl RoomRunner {
    async fn process_actions(&mut self) -> bool {
        let num_actions = self.action_receiver.len().min(MAX_USER_ACTIONS);
        let mut user_messages = Vec::with_capacity(num_actions);

        // Process pending actions
        self.action_receiver
            .recv_many(&mut user_messages, num_actions.max(1))
            .await;

        let mut new_messages = Vec::new();
        let mut notifications = Vec::new();
        let mut room_closed = false;

        for user_message in user_messages {
            // Handle join if user not in room
            if !self
                .room
                .participants()
                .iter()
                .any(|p| p.user_id == user_message.user_id)
            {
                self.room
                    .add_participant(user_message.user_id.clone(), user_message.country.clone());
                notifications.push(Notification {
                    message: format!("{} joined the room", user_message.user_id),
                });
            }

            let (message, action_notifications) = self.room.process_action(
                &user_message.user_id,
                &user_message.country,
                user_message.action,
            );

            if let Some(msg) = message {
                new_messages.push(msg);
            }
            notifications.extend(action_notifications);
        }

        // Check if room should close
        if self.room.is_empty() && !new_messages.is_empty() {
            room_closed = true;
        }

        // Create update for each connected participant's country
        // For simplicity, we broadcast a "neutral" state and let clients filter
        // In production, you might want per-country channels
        let room_state = self.room.get_censored_state_for(&"".to_string());
        let censored_new_messages: Vec<CensoredMessage> = new_messages
            .iter()
            .map(|msg| self.room.censor_message_for(msg, &"".to_string()))
            .collect();

        let update = RoomUpdate {
            room_state,
            new_messages: censored_new_messages,
            notifications,
            room_closed,
        };

        self.update_sender.send_replace(update);
        room_closed
    }

    fn run_in_background(mut self) {
        let room_id = self.room.room_id().clone();
        tokio::spawn(async move {
            loop {
                // Event-driven: wait for actions instead of tick-based loop
                if self.action_receiver.is_closed() {
                    break;
                }

                if self.process_actions().await {
                    self.room_manager.remove_room(&room_id);
                    break;
                }
            }
        });
    }
}

impl RoomManager {
    pub fn from_config(config: impl RoomConfig + 'static) -> Arc<Self> {
        Arc::new(RoomManager {
            active_rooms: Default::default(),
            config: Box::new(config),
        })
    }

    pub fn get_filter_config(&self) -> &'static FilterConfig {
        self.config.get_filter_config()
    }

    pub fn list_rooms(&self) -> Vec<RoomId> {
        self.active_rooms
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn connect_to_room<T: Into<RoomId>>(&self, room_id: T) -> Option<RoomConnector> {
        let room_id = room_id.into();
        Some(self.active_rooms.get(&room_id)?.value().clone())
    }

    pub fn create_and_run_room(self: Arc<Self>) -> RoomId {
        let room_id: RoomId = Alphanumeric.sample_string(&mut rand::rng(), 16);
        self.create_room_with_id(room_id)
    }

    pub fn create_room_with_id(self: Arc<Self>, room_id: RoomId) -> RoomId {
        // Check if room already exists
        if self.active_rooms.contains_key(&room_id) {
            eprintln!("Room {} already exists, returning existing room", &room_id);
            return room_id;
        }

        let room = self.config.init_room(room_id.clone());

        let (action_sender, action_receiver) = mpsc::channel(MAX_USER_ACTIONS);
        let (update_sender, update_receiver) = watch::channel(RoomUpdate {
            room_state: room.get_censored_state_for(&"".to_string()),
            new_messages: vec![],
            notifications: vec![],
            room_closed: false,
        });

        eprintln!("Created room {}", &room_id);

        let allowed_words = Arc::new(room.allowed_words.clone());
        let banned_words = Arc::new(room.filter.config.banned_words.clone());

        let room_runner = RoomRunner {
            room,
            action_receiver,
            update_sender,
            room_manager: Arc::clone(&self),
        };
        room_runner.run_in_background();

        let room_connector = RoomConnector {
            action_sender,
            update_receiver,
            allowed_words,
            banned_words,
        };
        self.active_rooms.insert(room_id.clone(), room_connector);
        room_id
    }

    fn remove_room(&self, room_id: &RoomId) {
        eprintln!("Room {} closed", room_id);
        self.active_rooms.remove(room_id);
    }
}
