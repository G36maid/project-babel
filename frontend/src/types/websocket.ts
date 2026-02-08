// WebSocket message types - matches backend/src/data.rs

export type RoomId = string;
export type UserId = string;
export type MessageId = number;
export type CountryCode = string;
export type Timestamp = number;

export const COUNTRY_NAMES: Record<string, string> = {
  A: "Alveria",
  B: "Brezna",
  C: "Corvistan",
  D: "Dravia",
};

export const COUNTRIES = Object.entries(COUNTRY_NAMES).map(([code, name]) => ({
  value: code as CountryCode,
  label: name,
}));

export function getCountryName(code: string): string {
  return COUNTRY_NAMES[code] || code;
}

export interface Message {
  id: MessageId;
  sender_id: UserId;
  sender_country: CountryCode;
  content: string;
  timestamp: Timestamp;
}

export interface CensoredMessage {
  id: MessageId;
  sender_id: UserId;
  content: string;
  was_censored: boolean;
  timestamp?: Timestamp;
}

export interface Participant {
  user_id: UserId;
  country: CountryCode;
  joined_at: Timestamp;
}

export interface RoomState {
  room_id: RoomId;
  participants: Participant[];
  recent_messages: CensoredMessage[];
}

export interface Notification {
  message: string;
}

export interface PlayerProgress {
  user_id: UserId;
  country: CountryCode;
  discovered_count: number;
  total_required: number;
  completed: boolean;
}

export interface VictoryState {
  achieved: boolean;
  player_progress: PlayerProgress[];
  unlocked_at: number | null;
}

export interface RoomUpdate {
  room_state: RoomState;
  new_messages: CensoredMessage[];
  notifications: Notification[];
  room_closed: boolean;
  victory: VictoryState | null;
}

// UserAction - matches backend UserAction enum (snake_case)
// System-level actions handled by the Room/RoomManager
export type SystemAction =
  | { send_message: string }
  | { send_message_array: string[] }
  | { leave_room: null };

// Game-specific actions delegated to GameEngine/GameRules
export type GameAction = { submit_notes: Record<string, string[]> };

// Transport layer envelope for user actions
export type UserAction = { system: SystemAction } | { game: GameAction };

// Connection states
export type ConnectionState =
  | "idle"
  | "connecting"
  | "connected"
  | "disconnected"
  | "error";

// Room words info - from /api/rooms/{id}/info
export interface RoomWordsInfo {
  allowed_words: string[];
  banned_words: Record<string, string[]>;
}
