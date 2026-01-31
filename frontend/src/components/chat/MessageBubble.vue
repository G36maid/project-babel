<script setup lang="ts">
import type { CensoredMessage } from '@/types/websocket'

const props = defineProps<{
  message: CensoredMessage
  isOwn: boolean
  playerCountry?: string
}>()

function formatTime(timestamp: number | undefined): string {
  if (!timestamp || timestamp <= 0) return ''
  const date = new Date(timestamp * 1000)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div :class="['flex w-full mb-2', isOwn ? 'justify-end' : 'justify-start']">
    <div
      :class="[
        'message-tail relative max-w-[420px] rounded-2xl px-3 py-1.5 shadow-sm text-base leading-snug min-w-[120px]',
        isOwn 
          ? 'outgoing bg-[var(--tg-message-out)] text-[var(--tg-message-text,white)] rounded-br-none' 
          : 'incoming bg-[var(--tg-message-in)] text-[var(--tg-text)] rounded-bl-none'
      ]"
    >
      <!-- Sender Info (only for others) -->
      <div v-if="!isOwn && playerCountry" class="text-sm text-[var(--tg-accent)] font-medium mb-0.5 whitespace-nowrap overflow-hidden text-ellipsis">
        {{ playerCountry }}
      </div>
      
      <!-- Message Content + Time Container -->
      <!-- Use inline-block logic or flex to wrap time naturally -->
      <div class="relative pr-9 pb-1">
        <span class="break-words whitespace-pre-wrap">{{ message.content }}</span>
        
        <!-- Censored Indicator -->
        <span v-if="message.was_censored" class="text-[var(--tg-destructive)] text-xs ml-1 italic">
          (censored)
        </span>

         <!-- Timestamp (Float bottom right) -->
        <div 
          :class="[
            'absolute bottom-0 right-[-4px] text-[11px] select-none',
            isOwn ? 'text-blue-100/70' : 'text-[var(--tg-text-secondary)]'
          ]"
        >
          {{ formatTime(message.timestamp) }}
          <!-- Optional read checkmarks could go here -->
        </div>
      </div>
    </div>
  </div>
</template>
