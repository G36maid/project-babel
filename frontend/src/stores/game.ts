import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useGameStore = defineStore('game', () => {
  const connected = ref(false)
  const messages = ref<any[]>([])
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
