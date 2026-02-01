<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useGameStore } from '@/stores/game'
import TelegramLayout from '@/components/layout/TelegramLayout.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import ChatArea from '@/components/layout/ChatArea.vue'
import ChatHeader from '@/components/chat/ChatHeader.vue'
import MessageList from '@/components/chat/MessageList.vue'
import ChatInput from '@/components/chat/ChatInput.vue'
import SymbolKeyboard from '@/components/symbols/SymbolKeyboard.vue'

const route = useRoute()
const router = useRouter()
const gameStore = useGameStore()

// Local state
const showSymbolKeyboard = ref(false)
const inputText = ref('')
const mobileSidebarOpen = ref(false)

// Computed
const messages = computed(() => gameStore.messages)
const roomState = computed(() => gameStore.roomState)
const connectionState = computed(() => gameStore.connectionState)
const playerId = computed(() => gameStore.playerId)
const playerName = computed(() => gameStore.playerName)

const participants = computed(() => {
  return roomState.value?.participants || []
})

const roomName = computed(() => {
  const roomId = route.params.roomId as string
  if (roomId) {
    return `Room: ${roomId}`
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
  const roomId = route.params.roomId as string
  gameStore.loadPlayerInfo()
  
  if (roomId) {
    gameStore.currentRoomId = roomId
    
    if (gameStore.playerToken) {
      gameStore.connect(roomId, gameStore.playerToken)
    } else {
      console.warn('No token found, redirecting to login')
      router.push({ name: 'home', query: { roomId } })
    }
  }
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
