<script setup lang="ts">
import { computed } from 'vue'
import { useSymbols } from '@/composables/useSymbols'
import { useGameStore } from '@/stores/game'
import SymbolRenderer from './SymbolRenderer.vue'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'select': [word: string]
  'close': []
}>()

const { getAllSymbols } = useSymbols()
const gameStore = useGameStore()

// Get all available symbols from composable, filtered by allowed words
const allSymbols = computed(() => {
  const symbolsRecord = getAllSymbols()
  const allSymbolWords = Object.keys(symbolsRecord)
  
  // If no allowed words are set yet, show all symbols (fallback behavior)
  if (gameStore.allowedWords.length === 0) {
    return allSymbolWords
  }
  
  // Filter symbols to only show allowed words
  return allSymbolWords.filter(word => gameStore.allowedWords.includes(word))
})

function onSymbolClick(word: string) {
  emit('select', word)
}

function onClose() {
  emit('close')
}
</script>

<template>
  <div 
    v-if="visible"
    class="bg-[var(--tg-bg-secondary)] border-t border-[var(--tg-bg-chat)]"
  >
    <div class="flex justify-between items-center py-1 px-2">
      <span class="text-[var(--tg-text-secondary)] text-xs">Symbols</span>
      <button 
        @click="onClose"
        class="text-[var(--tg-text-secondary)] hover:text-[var(--tg-text)] px-1.5 py-0.5 text-sm"
      >
        ✕
      </button>
    </div>
    
    <!-- Compact Grid with Max Height -->
    <div class="overflow-y-auto max-h-48 px-2 pb-2">
      <div class="grid grid-cols-8 gap-1">
        <button
          v-for="word in allSymbols"
          :key="word"
          @click="onSymbolClick(word)"
          class="aspect-square flex items-center justify-center bg-[var(--tg-bg)] hover:bg-[var(--tg-bg-chat)] rounded transition-colors"
          :title="word"
        >
          <SymbolRenderer :word="word" :size="20" />
        </button>
      </div>
    </div>
  </div>
</template>
