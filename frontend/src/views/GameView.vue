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
import NotebookPanel from '@/components/chat/NotebookPanel.vue'

const route = useRoute()
const router = useRouter()
const gameStore = useGameStore()

// Local state
const showSymbolKeyboard = ref(false)
const showNotebook = ref(false)
const inputText = ref('')
const mobileSidebarOpen = ref(false)

// Computed
const messages = computed(() => gameStore.messages)
const roomState = computed(() => gameStore.roomState)
const connectionState = computed(() => gameStore.connectionState)
const playerId = computed(() => gameStore.playerId)
const playerName = computed(() => gameStore.playerName)
const victoryState = computed(() => gameStore.victoryState)

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
  inputText.value += emoji + ' '
}

function returnToHome() {
  gameStore.cleanup()
  router.push({ name: 'home' })
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
      <div class="h-full flex flex-col">
        <!-- Participant List (top half) -->
        <div class="flex-1 overflow-y-auto">
          <Sidebar 
            :room-name="roomName"
            :participants="participants"
          />
        </div>
        
        <!-- Notebook Panel (bottom half) -->
        <div class="flex-1 border-t border-gray-700 overflow-y-auto">
          <div class="p-2">
            <NotebookPanel />
          </div>
        </div>
      </div>
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
          <!-- Victory Screen -->
          <div v-if="victoryState?.achieved" class="fixed inset-0 bg-black bg-opacity-80 flex items-center justify-center z-50">
            <div class="bg-gray-800 rounded-lg p-8 max-w-2xl w-full mx-4 shadow-2xl border border-green-500">
              <h2 class="text-3xl font-bold text-green-400 mb-4 text-center">🎉 Victory Achieved!</h2>
              <p class="text-gray-300 text-center mb-6">All countries have discovered all banned words!</p>
              
              <div class="space-y-3">
                <h3 class="text-lg font-semibold text-gray-200 mb-2">Player Progress:</h3>
                <div v-for="progress in victoryState.player_progress" :key="progress.user_id" 
                     class="bg-gray-700 rounded p-3 flex justify-between items-center">
                  <div>
                    <span class="font-semibold text-blue-400">Country {{ progress.country }}</span>
                    <span class="text-gray-400 ml-2">({{ progress.user_id }})</span>
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-gray-300">{{ progress.discovered_count }} / {{ progress.total_required }} words</span>
                    <span v-if="progress.completed" class="text-green-400 font-bold">✓</span>
                  </div>
                </div>
              </div>
              
              <button 
                @click="returnToHome"
                class="mt-6 w-full bg-green-600 hover:bg-green-700 text-white font-bold py-3 px-6 rounded-lg transition"
              >
                Return to Home
              </button>
            </div>
          </div>
          
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
