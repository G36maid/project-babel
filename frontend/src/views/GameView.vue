<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useGameStore } from '@/stores/game'

const gameStore = useGameStore()

// Local state
const showSymbolKeyboard = ref(false)
const inputText = ref('')
const mobileSidebarOpen = ref(false)
const playerId = ref(`player_${Math.random().toString(36).substr(2, 9)}`)

// Computed
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
  <TelegramLayout 
    :sidebar-open="mobileSidebarOpen"
    @close-sidebar="mobileSidebarOpen = false"
  >
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
            @open-sidebar="mobileSidebarOpen = true"
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
