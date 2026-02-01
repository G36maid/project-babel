<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { DynamicScroller, DynamicScrollerItem } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import type { CensoredMessage, Participant } from '@/types/websocket'
import { getCountryName } from '@/types/websocket'
import MessageBubble from './MessageBubble.vue'

const props = defineProps<{
  messages: CensoredMessage[]
  currentPlayerId: string
  participants: Participant[]
}>()

const scroller = ref<any>(null)

function isOwnMessage(message: CensoredMessage): boolean {
  return message.sender_id === props.currentPlayerId
}

function isSystemMessage(message: CensoredMessage): boolean {
  return message.sender_id === 'SYSTEM'
}

function getPlayerCountry(senderId: string): string {
  const participant = props.participants.find(p => p.user_id === senderId)
  return participant ? getCountryName(participant.country) : 'Unknown'
}

// Auto-scroll to bottom on new messages
watch(() => props.messages.length, async () => {
  await nextTick()
  if (scroller.value && scroller.value.scrollToBottom) {
    scroller.value.scrollToBottom()
  }
}, { flush: 'post' })
</script>

<template>
  <div class="h-full">
    <div v-if="messages.length === 0" class="flex h-full items-center justify-center text-[var(--tg-text-secondary)]">
      <p>No messages yet. Start the conversation!</p>
    </div>
    
    <DynamicScroller
      v-else
      ref="scroller"
      :items="messages"
      :min-item-size="60"
      key-field="id"
      class="message-list-scroller h-full"
    >
      <template #default="{ item, index, active }">
        <DynamicScrollerItem
          :item="item"
          :active="active"
          :data-index="index"
          :size-dependencies="[item.content]"
        >
          <!-- System Message -->
          <div v-if="isSystemMessage(item)" class="px-2 py-2 flex justify-center">
            <div class="text-sm text-[var(--tg-text-secondary)] text-center max-w-md px-3 py-1.5 bg-[var(--tg-bg-secondary)] rounded-lg">
              {{ item.content }}
            </div>
          </div>
          <!-- Regular Message -->
          <MessageBubble
            v-else
            :message="item"
            :is-own="isOwnMessage(item)"
            :player-country="getPlayerCountry(item.sender_id)"
            class="px-2 py-1"
          />
        </DynamicScrollerItem>
      </template>
    </DynamicScroller>
  </div>
</template>

<style scoped>
.message-list-scroller {
  overflow-y: auto;
}

/* Match scrollbar styling with Telegram theme */
.message-list-scroller::-webkit-scrollbar {
  width: 6px;
}

.message-list-scroller::-webkit-scrollbar-track {
  background: transparent;
}

.message-list-scroller::-webkit-scrollbar-thumb {
  background-color: var(--tg-text-secondary);
  border-radius: 3px;
  opacity: 0.5;
}

.message-list-scroller::-webkit-scrollbar-thumb:hover {
  background-color: var(--tg-text);
}
</style>
