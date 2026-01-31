<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useGameStore } from '@/stores/game'
import TelegramLayout from '@/components/layout/TelegramLayout.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import ChatArea from '@/components/layout/ChatArea.vue'
import ChatHeader from '@/components/chat/ChatHeader.vue'
import MessageList from '@/components/chat/MessageList.vue'
import ChatInput from '@/components/chat/ChatInput.vue'
import SymbolKeyboard from '@/components/symbols/SymbolKeyboard.vue'

const router = useRouter()
const gameStore = useGameStore()

// Local state
const showSymbolKeyboard = ref(false)
const inputText = ref('')
const playerId = ref(`player_${Math.random().toString(36).substr(2, 9)}`)

// Computed
const isConnected = computed(() => gameStore.connected)
const messages = computed(() => gameStore.messages)
const roomState = computed(() => gameStore.roomState)
const connectionState = computed(() => gameStore.connectionState)

const participants = computed(() => {
  return roomState.value?.participants || []
})

const roomName = computed(() => {
  if (roomState.value) {
    return `Room ${roomState.value.room_id}`
  }
  return 'Connecting...'
})

// Actions
function handleSend(content: string) {
  if (content.trim()) {
    gameStore.sendMessage(content.trim())
  }
}

function handleSymbolSelect(emoji: string) {
  inputText.value += emoji
}

function handleLeave() {
  gameStore.leaveRoom()
  router.push('/')
}

// Connect on mount
onMounted(() => {
  // For demo/development, auto-connect to a test room
  // In production, this would use proper room joining flow
  const token = 'test_token'
  const roomId = 'test_room'
  gameStore.connect(roomId, token)
})
</script>

<template>
  <TelegramLayout>
    <!-- Sidebar Slot -->
    <template #sidebar>
      <Sidebar 
        :room-name="roomName"
        :participants="participants"
      />
    </template>
    
    <!-- Chat Slot -->
    <template #chat>
      <ChatArea>
        <!-- Chat Header -->
        <template #header>
          <ChatHeader
            :room-name="roomName"
            :connection-state="connectionState"
          />
        </template>
        
        <!-- Messages -->
        <template #messages>
          <MessageList
            :messages="messages"
            :current-player-id="playerId"
            :participants="participants"
          />
        </template>
        
        <!-- Input -->
        <template #input>
          <div>
            <SymbolKeyboard
              :visible="showSymbolKeyboard"
              @select="handleSymbolSelect"
              @close="showSymbolKeyboard = false"
            />
            <ChatInput
              v-model="inputText"
              v-model:show-symbol-keyboard="showSymbolKeyboard"
              @send="handleSend"
            />
          </div>
        </template>
      </ChatArea>
    </template>
  </TelegramLayout>
</template>
