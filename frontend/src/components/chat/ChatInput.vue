<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  modelValue: string
  showSymbolKeyboard: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'update:showSymbolKeyboard': [value: boolean]
  'send': [content: string]
}>()

const canSend = computed(() => props.modelValue.trim().length > 0)

function onInput(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

function onSend() {
  if (canSend.value) {
    emit('send', props.modelValue.trim())
    emit('update:modelValue', '')
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    onSend()
  }
}

function toggleSymbolKeyboard() {
  emit('update:showSymbolKeyboard', !props.showSymbolKeyboard)
}
</script>

<template>
  <div class="bg-[var(--tg-bg-secondary)] p-2">
    <div class="flex items-center gap-2 bg-[var(--tg-bg)] rounded-full px-3 py-2">
      <!-- Symbol Keyboard Toggle -->
      <button
        @click="toggleSymbolKeyboard"
        :class="[
          'p-2 rounded-full transition-colors',
          showSymbolKeyboard 
            ? 'bg-telegram-accent text-white' 
            : 'text-[var(--tg-text-secondary)] hover:bg-[var(--tg-bg-secondary)]'
        ]"
        title="Toggle symbol keyboard"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
        </svg>
      </button>
      
      <!-- Text Input -->
      <input
        :value="modelValue"
        @input="onInput"
        @keydown="onKeydown"
        type="text"
        placeholder="Type a message..."
        class="flex-1 bg-transparent text-[var(--tg-text)] placeholder-[var(--tg-text-secondary)] outline-none min-w-0"
      />
      
      <!-- Send Button -->
      <button
        @click="onSend"
        :disabled="!canSend"
        :class="[
          'p-2 rounded-full transition-colors',
          canSend
            ? 'bg-telegram-accent text-white hover:bg-opacity-80'
            : 'text-[var(--tg-text-secondary)] cursor-not-allowed'
        ]"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
      </button>
    </div>
  </div>
</template>
