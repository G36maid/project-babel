<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useGameStore } from '@/stores/game'
import { apiClient } from '@/api/client'

const gameStore = useGameStore()

// Dynamic word counts per country (only for countries with players)
const wordCounts = ref<Record<string, number>>({})

// State for each country's guesses (dynamic based on actual banned word count)
const notes = ref<Record<string, string[]>>({})

const loading = ref(true)
const submitting = ref(false)
const submitResult = ref<{
  success: boolean
  discovered_count: number
  total_required: number
  victory_achieved: boolean
} | null>(null)

// Get countries that have active players
const activeCountries = computed(() => {
  if (!gameStore.roomState?.participants) return []
  return [...new Set(gameStore.roomState.participants.map(p => p.country))].sort()
})

const canSubmit = computed(() => {
  // Check if all fields for active countries are filled
  return activeCountries.value.every(country => {
    const countryNotes = notes.value[country]
    return countryNotes && countryNotes.every(note => note.trim().length > 0)
  })
})

const totalWords = computed(() => 
  Object.values(wordCounts.value).reduce((sum, count) => sum + count, 0)
)

// Fetch room info to get actual banned word counts
async function fetchRoomInfo() {
  try {
    loading.value = true
    
    // Wait a bit for room to be created if needed
    await new Promise(resolve => setTimeout(resolve, 500))
    
    const response: any = await apiClient.get(
      `/rooms/${gameStore.currentRoomId}/info`
    )
    
    console.log('[Notebook] Room info received:', response)
    
    // Update word counts based on actual banned words
    const bannedWords = response.banned_words as Record<string, string[]>
    const newNotes: Record<string, string[]> = {}
    const newCounts: Record<string, number> = {}
    
    // Only initialize notes for countries with active players
    for (const country of activeCountries.value) {
      if (bannedWords[country]) {
        const count = bannedWords[country].length
        newCounts[country] = count
        newNotes[country] = Array(count).fill('')
        console.log(`[Notebook] Country ${country}: ${count} words`)
      }
    }
    
    wordCounts.value = newCounts
    notes.value = newNotes
    console.log('[Notebook] Updated wordCounts:', newCounts)
  } catch (error: any) {
    console.error('Failed to fetch room info:', error)
    // If 404, room might not exist yet - retry once
    if (error.status === 404) {
      await new Promise(resolve => setTimeout(resolve, 1000))
      try {
        const response: any = await apiClient.get(
          `/rooms/${gameStore.currentRoomId}/info`
        )
        const bannedWords = response.banned_words as Record<string, string[]>
        const newNotes: Record<string, string[]> = {}
        const newCounts: Record<string, number> = {}
        for (const country of activeCountries.value) {
          if (bannedWords[country]) {
            newCounts[country] = bannedWords[country].length
            newNotes[country] = Array(bannedWords[country].length).fill('')
          }
        }
        wordCounts.value = newCounts
        notes.value = newNotes
      } catch (retryError) {
        console.error('Retry failed:', retryError)
      }
    }
  } finally {
    loading.value = false
  }
}

// Watch for participant changes and update notes accordingly
watch(activeCountries, (newCountries, oldCountries) => {
  if (JSON.stringify(newCountries) === JSON.stringify(oldCountries)) return
  
  console.log('[Notebook] Active countries changed:', newCountries)
  
  // Refetch room info when participants change
  if (gameStore.currentRoomId && newCountries.length > 0) {
    fetchRoomInfo()
  }
})

onMounted(() => {
  console.log('[Notebook] onMounted, currentRoomId:', gameStore.currentRoomId)
  if (gameStore.currentRoomId) {
    fetchRoomInfo()
  } else {
    loading.value = false
  }
})

async function submitNotes() {
  if (!canSubmit.value) {
    alert('Please fill in all fields')
    return
  }

  submitting.value = true
  submitResult.value = null

  try {
    const payload = {
      notes: Object.fromEntries(
        Object.entries(notes.value)
          .filter(([country]) => activeCountries.value.includes(country))
          .map(([country, words]) => [
          country,
          words.map(w => w.trim())
        ])
      )
    }

    const response: any = await apiClient.post(
      `/rooms/${gameStore.currentRoomId}/submit_notes`,
      payload,
      {
        'X-User-Token': gameStore.playerToken
      }
    )

    submitResult.value = response
    
    if (response.victory_achieved) {
      alert('🎉 Congratulations! All players have discovered all banned words!')
    } else {
      alert(`Submitted! You've discovered ${response.discovered_count} out of ${response.total_required} words.`)
    }
  } catch (error) {
    console.error('Failed to submit notes:', error)
    alert('Failed to submit notes. Please try again.')
  } finally {
    submitting.value = false
  }
}

function clearNotes() {
  const cleared: Record<string, string[]> = {}
  for (const [country, count] of Object.entries(wordCounts.value)) {
    cleared[country] = Array(count).fill('')
  }
  notes.value = cleared
  submitResult.value = null
}
</script>

<template>
  <div class="notebook-panel bg-gray-800 rounded-lg p-4 shadow-lg">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-lg font-bold text-blue-400">📝 Banned Words Notebook</h3>
      <button
        @click="clearNotes"
        :disabled="loading"
        class="text-sm text-gray-400 hover:text-gray-200 disabled:opacity-50 transition"
      >
        Clear All
      </button>
    </div>

    <div v-if="loading" class="text-center py-8 text-gray-400">
      Loading...
    </div>

    <div v-else-if="activeCountries.length === 0" class="text-center py-8 text-gray-400">
      Waiting for players to join...
    </div>

    <div v-else class="text-sm text-gray-400 mb-4">
      Guess {{ totalWords }} banned words for {{ activeCountries.length }} active {{ activeCountries.length === 1 ? 'country' : 'countries' }}:
    </div>

    <div v-if="!loading && activeCountries.length > 0" class="space-y-4">
      <div v-for="country in activeCountries" :key="country" class="country-section">
        <div class="flex items-center mb-2">
          <div class="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center font-bold text-white mr-3">
            {{ country }}
          </div>
          <span class="text-gray-300 font-semibold">
            Country {{ country }} ({{ wordCounts[country] || 1 }} {{ wordCounts[country] === 1 ? 'word' : 'words' }})
          </span>
        </div>
        
        <div :class="[
          'grid gap-2 ml-11',
          wordCounts[country] === 2 ? 'grid-cols-2' : 'grid-cols-3'
        ]">
          <input
            v-for="i in wordCounts[country]"
            :key="i"
            v-model="(notes as any)[country][i - 1]"
            type="text"
            :placeholder="`Word ${i}`"
            class="bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition"
          />
        </div>
      </div>
    </div>

    <div v-if="submitResult" class="mt-4 p-3 rounded bg-gray-700">
      <div class="text-sm text-gray-300">
        <span class="font-semibold text-green-400">Progress:</span>
        {{ submitResult.discovered_count }} / {{ submitResult.total_required }} words discovered
      </div>
      <div v-if="submitResult.victory_achieved" class="text-green-400 font-bold mt-1">
        🎉 Victory Achieved!
      </div>
    </div>

    <div v-if="!loading && activeCountries.length > 0" class="flex gap-3 mt-6">
      <button
        @click="submitNotes"
        :disabled="!canSubmit || submitting || activeCountries.length === 0"
        class="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white font-bold py-3 px-6 rounded-lg transition"
      >
        {{ submitting ? 'Submitting...' : 'Submit Guesses' }}
      </button>
    </div>

    <div v-if="!loading && activeCountries.length > 0" class="text-xs text-gray-500 mt-3 text-center">
      When all {{ activeCountries.length }} {{ activeCountries.length === 1 ? 'player discovers' : 'players discover' }} all {{ totalWords }} banned words, the game is won!
    </div>
  </div>
</template>

<style scoped>
.notebook-panel {
  max-height: 80vh;
  overflow-y: auto;
}
</style>
