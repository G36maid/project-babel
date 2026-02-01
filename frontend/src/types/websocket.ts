// WebSocket message types - matches backend/src/data.rs

export type RoomId = string
export type UserId = string
export type MessageId = number
export type CountryCode = string
export type Timestamp = number

export const COUNTRY_NAMES: Record<string, string> = {
  'A': 'Veridia',
  'B': 'Aethelgard',
  'C': 'Orynthia',
  'D': 'Kaelis'
}

export const COUNTRIES = Object.entries(COUNTRY_NAMES).map(([code, name]) => ({
  value: code as CountryCode,
  label: name
}))

export function getCountryName(code: string): string {
  return COUNTRY_NAMES[code] || code
}

export interface Message {
  id: MessageId
  sender_id: UserId
  sender_country: CountryCode
  content: string
  timestamp: Timestamp
}

export interface CensoredMessage {
  id: MessageId
  sender_id: UserId
  content: string
  was_censored: boolean
  timestamp?: Timestamp
}

export interface Participant {
  user_id: UserId
  country: CountryCode
  joined_at: Timestamp
}

export interface RoomState {
  room_id: RoomId
  participants: Participant[]
  recent_messages: CensoredMessage[]
}

export interface Notification {
  message: string
}

export interface RoomUpdate {
  room_state: RoomState
  new_messages: CensoredMessage[]
  notifications: Notification[]
  room_closed: boolean
}

// UserAction - matches backend UserAction enum (snake_case)
export type UserAction =
  | { send_message: string }
  | { leave_room: null }

// Connection states
export type ConnectionState = 'idle' | 'connecting' | 'connected' | 'disconnected' | 'error'
