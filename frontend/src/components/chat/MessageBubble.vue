<script setup lang="ts">
import { computed } from 'vue'
import type { CensoredMessage } from '@/types/websocket'
import SymbolRenderer from '@/components/symbols/SymbolRenderer.vue'

const props = defineProps<{
  message: CensoredMessage
  isOwn: boolean
  playerCountry?: string
  playerName?: string
}>()

function formatTime(timestamp: number | undefined): string {
  if (!timestamp || timestamp <= 0) return ''
  const date = new Date(timestamp * 1000)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

// Parse message content into word tokens
const tokens = computed(() => {
  return props.message.content.split(' ')
})
</script>

<template>
  <div :class="['flex w-full mb-2', isOwn ? 'justify-end' : 'justify-start']">
    <div
      :class="[
        'message-tail relative max-w-[420px] rounded-2xl px-3 py-1.5 shadow-sm text-base leading-snug min-w-[120px]',
        isOwn 
          ? 'outgoing bg-[var(--tg-message-out)] text-[var(--tg-message-text-out)] rounded-br-none' 
          : 'incoming bg-[var(--tg-message-in)] text-[var(--tg-text)] rounded-bl-none'
      ]"
    >
      <!-- Sender Info (only for others) -->
      <div v-if="!isOwn && (playerName || playerCountry)" class="text-sm text-[var(--tg-accent)] font-medium mb-0.5 whitespace-nowrap overflow-hidden text-ellipsis">
        <template v-if="playerName && playerCountry">{{ playerName }} ({{ playerCountry }})</template>
        <template v-else>{{ playerName || playerCountry }}</template>
      </div>
      
      <!-- Message Content + Time Container -->
      <!-- Use inline-block logic or flex to wrap time naturally -->
      <div class="relative pr-9 pb-1">
        <span class="break-words whitespace-normal">
          <template v-for="(token, index) in tokens" :key="index">
            <SymbolRenderer :word="token" :size="20" />
            <span v-if="index < tokens.length - 1"> </span>
          </template>
        </span>
        
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
