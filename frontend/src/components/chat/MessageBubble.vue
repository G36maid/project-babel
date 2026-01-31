<script setup lang="ts">
import type { CensoredMessage } from '@/types/websocket'

const props = defineProps<{
  message: CensoredMessage
  isOwn: boolean
  playerCountry?: string
}>()

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div :class="['flex w-full px-2 py-1', isOwn ? 'justify-end' : 'justify-start']">
    <div
      :class="[
        'max-w-[70%] rounded-xl px-3 py-2 relative',
        isOwn 
          ? 'bg-telegram-message-out text-white rounded-br-md' 
          : 'bg-telegram-message-in text-[var(--tg-text)] rounded-bl-md'
      ]"
    >
      <!-- Sender Info (only for others) -->
      <div v-if="!isOwn && playerCountry" class="text-xs text-telegram-accent mb-1 font-medium">
        {{ playerCountry }}
      </div>
      
      <!-- Message Content -->
      <div class="text-sm break-words">
        {{ message.content }}
      </div>
      
      <!-- Censored Indicator -->
      <div v-if="message.was_censored" class="text-xs text-yellow-400 mt-1 italic">
        *** censored ***
      </div>
      
      <!-- Timestamp -->
      <div 
        :class="[
          'text-xs mt-1 text-right',
          isOwn ? 'text-blue-200' : 'text-[var(--tg-text-secondary)]'
        ]"
      >
        {{ formatTime(message.id) }}
      </div>
    </div>
  </div>
</template>
