import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useWebSocket } from '@vueuse/core'
import type { RoomUpdate, UserAction, ConnectionState, CensoredMessage } from '@/types/websocket'

interface LoginResponse {
  token: string
}

const TEST_ROOM_ID = 'test_room'

export const useGameStore = defineStore('game', () => {
  // State
  const connected = ref(false)
  const connectionState = ref<ConnectionState>('idle')
  const messages = ref<CensoredMessage[]>([])
  const playerId = ref('')
  const playerName = ref('')
  const playerToken = ref('')
  const roomState = ref<RoomUpdate['room_state'] | null>(null)
  const notifications = ref<string[]>([])

  // WebSocket instance (will be set in connect)
  let ws: ReturnType<typeof useWebSocket> | null = null

  function connect(roomId: string, token: string) {
    console.log('[WebSocket] connect() called', { roomId, token: token.substring(0, 10) + '...' })
    
    if (ws) {
      console.warn('[WebSocket] Already initialized, close first')
      return
    }

    connectionState.value = 'connecting'
    console.log('[WebSocket] State: connecting')

    // Determine WebSocket URL based on environment
    let wsUrl: string
    if (import.meta.env.DEV) {
      // Development: connect directly to backend
      wsUrl = `ws://localhost:3000/api/rooms/${roomId}/connect?token=${token}`
    } else {
      // Production: use same host as frontend
      wsUrl = `ws://${window.location.host}/api/rooms/${roomId}/connect?token=${token}`
    }
    console.log('[WebSocket] Connecting to:', wsUrl)

    ws = useWebSocket(wsUrl, {
      autoReconnect: {
        retries: 3,
        delay: 1000,
        onFailed() {
          console.error('[WebSocket] Auto-reconnect failed after 3 retries')
          connectionState.value = 'error'
        }
      },
      onConnected() {
        console.log('[WebSocket] ✅ Connected successfully!')
        connected.value = true
        connectionState.value = 'connected'
      },
      onDisconnected() {
        console.log('[WebSocket] ❌ Disconnected')
        connected.value = false
        connectionState.value = 'disconnected'
      },
      onError(_ws, event) {
        console.error('[WebSocket] ❌ Error:', event)
        connectionState.value = 'error'
      },
      onMessage(_ws, event) {
        console.log('[WebSocket] 📨 Received message:', event.data)
        try {
          const data = JSON.parse(event.data) as RoomUpdate
          console.log('[WebSocket] Parsed data:', data)
          roomState.value = data.room_state
          messages.value.push(...data.new_messages)
          notifications.value.push(...data.notifications.map(n => n.message))

          if (data.room_closed) {
            console.log('[WebSocket] Room closed by server')
            connectionState.value = 'disconnected'
          }
        } catch (err) {
          console.error('[WebSocket] Failed to parse message:', err)
        }
      }
    })
    
    console.log('[WebSocket] WebSocket instance created')
  }

  function sendMessage(content: string) {
    if (!ws || !ws.send) {
      console.error('[Store] WebSocket not connected')
      return
    }

    console.log('[Store] Sending message:', content)
    const action: UserAction = { send_message: content }
    const payload = JSON.stringify(action)
    console.log('[Store] Payload:', payload)
    ws.send(payload)
  }

  function leaveRoom() {
    if (!ws || !ws.send) return

    const action: UserAction = { leave_room: null }
    ws.send(JSON.stringify(action))
    ws.close()
    ws = null
    connected.value = false
    connectionState.value = 'idle'
  }

  function cleanup() {
    if (ws) {
      ws.close()
      ws = null
    }
    connected.value = false
    connectionState.value = 'idle'
    messages.value = []
    roomState.value = null
    notifications.value = []
  }

  function setPlayerInfo(name: string, token: string) {
    playerName.value = name
    playerToken.value = token
    playerId.value = name
    // Store in localStorage for persistence
    localStorage.setItem('babel_player_name', name)
    localStorage.setItem('babel_player_token', token)
  }

  async function login(username: string, country: string): Promise<string> {
    try {
      const response = await fetch('/api/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, country })
      })

      if (!response.ok) {
        throw new Error('Login failed')
      }

      const data: LoginResponse = await response.json()

      // Update local state
      playerName.value = username
      playerToken.value = data.token
      playerId.value = username

      // Persist
      localStorage.setItem('babel_player_name', username)
      localStorage.setItem('babel_player_token', data.token)

      return data.token
    } catch (err) {
      console.error('[Store] Login error:', err)
      throw err
    }
  }

  function loadPlayerInfo() {
    const savedName = localStorage.getItem('babel_player_name')
    const savedToken = localStorage.getItem('babel_player_token')
    if (savedName && savedToken) {
      playerName.value = savedName
      playerToken.value = savedToken
      playerId.value = savedName
    }
  }

  async function createRoom(token: string): Promise<string> {
    const response = await fetch('/api/rooms', {
      method: 'POST',
      headers: {
        'X-User-Token': token
      }
    })

    if (!response.ok) {
      throw new Error('Failed to create room')
    }

    return await response.text()
  }

  async function ensureTestRoom(_token: string): Promise<void> {
    // No need to check - backend will auto-create test_room on first connection
    console.log('Test room will be auto-created on connection')
  }

  function connectToTestRoom() {
    console.log('[Store] connectToTestRoom() called')
    console.log('[Store] Current playerToken:', playerToken.value)
    
    if (!playerToken.value) {
      console.log('[Store] No token, loading from localStorage...')
      loadPlayerInfo()
    }
    
    if (playerToken.value) {
      console.log('[Store] Connecting to test room with token:', playerToken.value.substring(0, 10) + '...')
      connect(TEST_ROOM_ID, playerToken.value)
    } else {
      console.error('[Store] No player token available! Cannot connect.')
    }
  }

  return {
    connected,
    connectionState,
    messages,
    playerId,
    playerName,
    playerToken,
    roomState,
    notifications,
    connect,
    sendMessage,
    leaveRoom,
    cleanup,
    setPlayerInfo,
    loadPlayerInfo,
    createRoom,
    ensureTestRoom,
    connectToTestRoom,
    login
  }
})
