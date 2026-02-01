<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useGameStore } from '@/stores/game'
import { COUNTRIES } from '@/types/websocket'

const router = useRouter()
const route = useRoute()
const gameStore = useGameStore()

const playerName = ref('')
const roomId = ref(gameStore.generateRoomId())
const selectedCountry = ref('A')
const isJoining = ref(false)
const errorMessage = ref('')

onMounted(() => {
  if (route.query.roomId) {
    roomId.value = route.query.roomId as string
  }
})

const countries = COUNTRIES

async function joinGame() {
  if (!playerName.value.trim()) {
    errorMessage.value = 'Please enter your name'
    return
  }

  if (!roomId.value.trim()) {
    errorMessage.value = 'Please enter a room ID'
    return
  }

  isJoining.value = true
  errorMessage.value = ''

  try {
    // Perform login
    const token = await gameStore.login(playerName.value.trim(), selectedCountry.value)
    
    // Set the room ID in the store
    gameStore.currentRoomId = roomId.value.trim()
    
    // Navigate to game
    router.push(`/game/${gameStore.currentRoomId}`)
  } catch (error) {
    errorMessage.value = 'Failed to join room. Please try again.'
    console.error('Join error:', error)
  } finally {
    isJoining.value = false
  }
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 p-4">
    <div class="w-full max-w-md">
      <h1 class="mb-4 text-center text-6xl font-bold tracking-tight text-white">
        🗼 Project Babel
      </h1>
      <p class="mb-8 text-center text-lg text-gray-400">
        In a world of surveillance, four strangers use a forgotten language to speak the truth.
      </p>

      <!-- Join Form -->
      <div class="rounded-2xl bg-gray-800/50 p-6 shadow-xl backdrop-blur-sm border border-gray-700">
        <h2 class="mb-6 text-2xl font-semibold text-white">Join Test Room</h2>
        
        <!-- Player Name Input -->
        <div class="mb-4">
          <label class="mb-2 block text-sm font-medium text-gray-300">
            Your Name
          </label>
          <input
            v-model="playerName"
            type="text"
            placeholder="Enter your name..."
            class="w-full rounded-lg border border-gray-600 bg-gray-700 px-4 py-3 text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50"
            @keydown.enter="joinGame"
          />
        </div>

        <!-- Room ID Input -->
        <div class="mb-4">
          <label class="mb-2 block text-sm font-medium text-gray-300">
            Room ID
          </label>
          <div class="flex gap-2">
            <input
              v-model="roomId"
              type="text"
              placeholder="Enter room ID..."
              class="w-full rounded-lg border border-gray-600 bg-gray-700 px-4 py-3 text-white placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50"
              @keydown.enter="joinGame"
            />
            <button 
              @click="roomId = gameStore.generateRoomId()"
              class="px-4 py-2 bg-gray-700 hover:bg-gray-600 border border-gray-600 rounded-lg text-gray-300 transition-colors"
              title="Generate random ID"
            >
              🔄
            </button>
          </div>
        </div>

        <!-- Token Selection -->
        <div class="mb-6">
          <label class="mb-2 block text-sm font-medium text-gray-300">
            Select Country
          </label>
          <select
            v-model="selectedCountry"
            class="w-full rounded-lg border border-gray-600 bg-gray-700 px-4 py-3 text-white focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50"
          >
            <option v-for="country in countries" :key="country.value" :value="country.value">
              {{ country.label }}
            </option>
          </select>
        </div>

        <!-- Error Message -->
        <div v-if="errorMessage" class="mb-4 rounded-lg bg-red-500/10 border border-red-500/50 px-4 py-3 text-red-400 text-sm">
          {{ errorMessage }}
        </div>

        <!-- Join Button -->
        <button
          @click="joinGame"
          :disabled="isJoining || !playerName.trim()"
          class="w-full rounded-lg bg-blue-600 px-6 py-3 text-lg font-bold text-white transition-all hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-800 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {{ isJoining ? 'Joining...' : 'Join Game' }}
        </button>
      </div>

      <!-- Instructions -->
      <div class="mt-6 rounded-lg bg-gray-800/30 p-4 text-sm text-gray-400">
        <p class="mb-2"><strong class="text-gray-300">How to play:</strong></p>
        <ul class="list-disc list-inside space-y-1">
          <li>Enter your name and select a country</li>
          <li>Share the <strong class="text-gray-300">Room ID</strong> with friends to play together</li>
          <li>Use symbols (0-25) to communicate</li>
          <li>Discover which words are censored in each country</li>
        </ul>
      </div>
    </div>
  </div>
</template>
