<script setup lang="ts">
const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'select': [emoji: string]
  'close': []
}>()

// 26 symbols - 4x7 grid (28 cells, last 2 are empty/function keys)
const symbols = [
  { emoji: '🔴', id: 'symbol-1' },
  { emoji: '🔵', id: 'symbol-2' },
  { emoji: '🟢', id: 'symbol-3' },
  { emoji: '🟡', id: 'symbol-4' },
  { emoji: '⭐', id: 'symbol-5' },
  { emoji: '❤️', id: 'symbol-6' },
  { emoji: '💎', id: 'symbol-7' },
  { emoji: '🌟', id: 'symbol-8' },
  { emoji: '⚡', id: 'symbol-9' },
  { emoji: '🔥', id: 'symbol-10' },
  { emoji: '🌈', id: 'symbol-11' },
  { emoji: '☀️', id: 'symbol-12' },
  { emoji: '🌙', id: 'symbol-13' },
  { emoji: '💧', id: 'symbol-14' },
  { emoji: '🔮', id: 'symbol-15' },
  { emoji: '💰', id: 'symbol-16' },
  { emoji: '🎵', id: 'symbol-17' },
  { emoji: '🎨', id: 'symbol-18' },
  { emoji: '🎲', id: 'symbol-19' },
  { emoji: '🎁', id: 'symbol-20' },
  { emoji: '📚', id: 'symbol-21' },
  { emoji: '⚔️', id: 'symbol-22' },
  { emoji: '🛡️', id: 'symbol-23' },
  { emoji: '🔑', id: 'symbol-24' },
  { emoji: '⚓', id: 'symbol-25' },
  { emoji: '🌿', id: 'symbol-26' },
]

function onSymbolClick(emoji: string) {
  emit('select', emoji)
}

function onClose() {
  emit('close')
}
</script>

<template>
  <div 
    v-if="visible"
    class="bg-[var(--tg-bg-secondary)] border-t border-[var(--tg-bg-chat)] p-2"
  >
    <div class="flex justify-between items-center mb-2 px-1">
      <span class="text-[var(--tg-text-secondary)] text-sm">Symbol Keyboard</span>
      <button 
        @click="onClose"
        class="text-[var(--tg-text-secondary)] hover:text-[var(--tg-text)] px-2 py-1 rounded"
      >
        ✕
      </button>
    </div>
    
    <!-- 4x7 Grid -->
    <div class="grid grid-cols-7 gap-1">
      <button
        v-for="symbol in symbols"
        :key="symbol.id"
        @click="onSymbolClick(symbol.emoji)"
        class="aspect-square flex items-center justify-center text-2xl bg-[var(--tg-bg)] hover:bg-[var(--tg-bg-chat)] rounded transition-colors min-h-[44px] min-w-[44px]"
        :title="symbol.id"
      >
        {{ symbol.emoji }}
      </button>
    </div>
  </div>
</template>
