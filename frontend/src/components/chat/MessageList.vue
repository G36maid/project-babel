<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { CensoredMessage, Participant } from '@/types/websocket'
import MessageBubble from './MessageBubble.vue'

const props = defineProps<{
  messages: CensoredMessage[]
  currentPlayerId: string
  participants: Participant[]
}>()

const messagesContainer = ref<HTMLElement | null>(null)

function isOwnMessage(message: CensoredMessage): boolean {
  return message.sender_id === props.currentPlayerId
}

function getPlayerCountry(senderId: string): string {
  const participant = props.participants.find(p => p.user_id === senderId)
  return participant?.country || 'Unknown'
}

// Auto-scroll to bottom on new messages
watch(() => props.messages.length, async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}, { flush: 'post' })
</script>

<template>
  <div 
    ref="messagesContainer"
    class="flex flex-col h-full overflow-y-auto p-2 space-y-1"
  >
    <div v-if="messages.length === 0" class="flex-1 flex items-center justify-center text-[var(--tg-text-secondary)]">
      <p>No messages yet. Start the conversation!</p>
    </div>
    
    <MessageBubble
      v-for="message in messages"
      :key="message.id"
      :message="message"
      :is-own="isOwnMessage(message)"
      :player-country="getPlayerCountry(message.sender_id)"
    />
  </div>
</template>
