<script setup lang="ts">
import type { Participant } from '@/types/websocket'
import { getCountryName } from '@/types/websocket'

const props = defineProps<{
  roomName?: string
  participants: Participant[]
}>()
</script>

<template>
  <div class="flex flex-col h-full bg-[var(--tg-bg-secondary)] border-r border-[var(--tg-bg-chat)]">
    <!-- Header -->
    <div class="h-14 flex items-center px-4 border-b border-[var(--tg-bg-chat)] bg-[var(--tg-bg)]">
      <h2 class="text-[var(--tg-text)] font-semibold text-lg">
        {{ roomName || 'Game Room' }}
      </h2>
    </div>
    
    <!-- Player List -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="participant in participants"
        :key="participant.user_id"
        class="flex items-center px-4 py-3 hover:bg-[var(--tg-bg)] cursor-pointer transition-colors"
      >
        <!-- Avatar -->
        <div class="w-10 h-10 rounded-full bg-telegram-accent flex items-center justify-center text-white font-semibold mr-3">
          {{ participant.user_id.slice(0, 2).toUpperCase() }}
        </div>
        
        <!-- Info -->
        <div class="flex-1 min-w-0">
          <div class="text-[var(--tg-text)] font-medium truncate">
            Player {{ participant.user_id.slice(-4) }}
          </div>
          <div class="text-[var(--tg-text-secondary)] text-sm">
            {{ getCountryName(participant.country) }}
          </div>
        </div>
      </div>
      
      <!-- Empty state -->
      <div v-if="participants.length === 0" class="p-4 text-[var(--tg-text-secondary)] text-center">
        No players connected
      </div>
    </div>
  </div>
</template>
