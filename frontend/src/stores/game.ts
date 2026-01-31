import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useWebSocket } from '@vueuse/core'
import type { RoomUpdate, UserAction, ConnectionState, CensoredMessage } from '@/types/websocket'

export const useGameStore = defineStore('game', () => {
  // State
  const connected = ref(false)
  const connectionState = ref<ConnectionState>('idle')
  const messages = ref<CensoredMessage[]>([])
  const playerId = ref('')
  const roomState = ref<RoomUpdate['room_state'] | null>(null)
  const notifications = ref<string[]>([])

  // WebSocket instance (will be set in connect)
  let ws: ReturnType<typeof useWebSocket> | null = null

  function connect(roomId: string, token: string) {
    if (ws) {
      console.warn('WebSocket already initialized, close first')
      return
    }

    connectionState.value = 'connecting'

    // Token passed as query parameter (browser WebSocket can't set headers)
    const wsUrl = `ws://${window.location.host}/api/rooms/${roomId}/connect?token=${token}`

    ws = useWebSocket(wsUrl, {
      autoReconnect: {
        retries: 3,
        delay: 1000,
        onFailed() {
          connectionState.value = 'error'
        }
      },
      onConnected() {
        connected.value = true
        connectionState.value = 'connected'
      },
      onDisconnected() {
        connected.value = false
        connectionState.value = 'disconnected'
      },
      onError() {
        connectionState.value = 'error'
      },
      onMessage(_ws, event) {
        try {
          const data = JSON.parse(event.data) as RoomUpdate
          roomState.value = data.room_state
          messages.value.push(...data.new_messages)
          notifications.value.push(...data.notifications.map(n => n.message))

          if (data.room_closed) {
            connectionState.value = 'disconnected'
          }
        } catch (err) {
          console.error('Failed to parse message:', err)
        }
      }
    })
  }

  function sendMessage(content: string) {
    if (!ws || !ws.send) {
      console.error('WebSocket not connected')
      return
    }

    const action: UserAction = { SendMessage: content }
    ws.send(JSON.stringify(action))
  }

  function leaveRoom() {
    if (!ws || !ws.send) return

    const action: UserAction = { LeaveRoom: null }
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

  return {
    connected,
    connectionState,
    messages,
    playerId,
    roomState,
    notifications,
    connect,
    sendMessage,
    leaveRoom,
    cleanup
  }
})
