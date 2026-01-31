import { defineStore } from 'pinia'
import { ref } from 'vue'

type GameMessage = {
  id: string
  playerId: string
  content: string
  timestamp: number
}

export const useGameStore = defineStore('game', () => {
  const connected = ref(false)
  const messages = ref<GameMessage[]>([])
  const playerId = ref('')

  // Placeholder actions - will be implemented with socket.io
  function connect() {
    // TODO: Implement socket.io connection
    console.log('Connecting to game server...')
  }

  function sendMessage(message: string) {
    // TODO: Implement socket.io send
    console.log('Sending message:', message)
  }

  return {
    connected,
    messages,
    playerId,
    connect,
    sendMessage,
  }
})
