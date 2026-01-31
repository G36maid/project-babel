<script setup lang="ts">
import type { ConnectionState } from '@/types/websocket'
import { useTheme } from '@/composables/useTheme'

const props = defineProps<{
  roomName?: string
  connectionState: ConnectionState
}>()

const { isDark, toggleTheme } = useTheme()

const connectionColor = {
  'idle': 'bg-gray-500',
  'connecting': 'bg-yellow-500',
  'connected': 'bg-green-500',
  'disconnected': 'bg-red-500',
  'error': 'bg-red-600'
}[props.connectionState]
</script>

<template>
  <div class="h-14 flex items-center justify-between px-4 bg-[var(--tg-bg-secondary)] border-b border-[var(--tg-bg-chat)]">
    <!-- Left: Room Name -->
    <div class="flex items-center">
      <h3 class="text-[var(--tg-text)] font-semibold">
        {{ roomName || 'Chat Room' }}
      </h3>
    </div>
    
    <!-- Right: Connection Status + Theme Toggle -->
    <div class="flex items-center gap-3">
      <!-- Connection Status -->
      <div class="flex items-center gap-2">
        <span class="text-[var(--tg-text-secondary)] text-sm capitalize">{{ connectionState }}</span>
        <div :class="['w-2.5 h-2.5 rounded-full', connectionColor]"></div>
      </div>
      
      <!-- Theme Toggle -->
      <button
        @click="toggleTheme"
        class="p-2 rounded-full hover:bg-[var(--tg-bg)] transition-colors text-[var(--tg-text-secondary)]"
        :title="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
      >
        <span v-if="isDark">☀️</span>
        <span v-else>🌙</span>
      </button>
    </div>
  </div>
</template>
