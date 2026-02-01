<script setup lang="ts">
import { computed } from "vue";

const modelValue = defineModel<string>("modelValue", { required: true });
const showSymbolKeyboard = defineModel<boolean>("showSymbolKeyboard", {
  required: true,
});

const emit = defineEmits<{
  send: [content: string];
}>();

const canSend = computed(() => modelValue.value.trim().length > 0);

function onSend() {
  if (canSend.value) {
    emit("send", modelValue.value.trim());
    modelValue.value = "";
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    onSend();
  }
}

function toggleSymbolKeyboard() {
  showSymbolKeyboard.value = !showSymbolKeyboard.value;
}
</script>

<template>
  <div class="bg-[var(--tg-bg-secondary)] px-2 py-2 flex items-center gap-2 border-t border-[var(--tg-bg-chat)]">
    <!-- Attachment Button (Paperclip) -->
    <button class="p-2 text-[var(--tg-text-secondary)] hover:text-[var(--tg-text)] transition-colors flex-shrink-0">
       <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M15.5 10C15.5 6.96243 13.0376 4.5 10 4.5C6.96243 4.5 4.5 6.96243 4.5 10V16.5C4.5 18.9853 6.51472 21 9 21C11.4853 21 13.5 18.9853 13.5 16.5V9.5C13.5 8.39543 12.6046 7.5 11.5 7.5C10.3954 7.5 9.5 8.39543 9.5 9.5V15.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>

    <!-- Input Area -->
    <div class="flex-1 min-w-0 bg-[var(--tg-bg)] rounded-3xl flex items-center px-4 py-3 shadow-sm">
      <input
        v-model="modelValue"
        @keydown="onKeydown"
        type="text"
        placeholder="Message..."
        class="flex-1 bg-transparent text-[var(--tg-text)] placeholder-[var(--tg-text-secondary)] outline-none min-w-0 text-[16px]"
      />
      
      <!-- Symbol/Emoji Toggle -->
      <button
        @click="toggleSymbolKeyboard"
        :class="[
          'ml-2 p-1 transition-colors',
          showSymbolKeyboard 
            ? 'text-[var(--tg-accent)]' 
            : 'text-[var(--tg-text-secondary)] hover:text-[var(--tg-text)]'
        ]"
      >
        <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor">
           <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </button>
    </div>
    
    <!-- Send / Mic Button -->
    <button
      @click="onSend"
      class="p-3 rounded-full transition-all transform active:scale-95 flex items-center justify-center flex-shrink-0"
      :class="[
        canSend 
          ? 'bg-[var(--tg-accent)] text-white shadow-md' 
          : 'bg-transparent text-[var(--tg-text-secondary)] hover:bg-[var(--tg-bg-secondary)]'
      ]"
    >
      <!-- Send Plane -->
      <svg v-if="canSend" class="w-6 h-6 ml-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
      </svg>
      <!-- Mic Icon (if cannot send) -->
      <svg v-else class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
      </svg>
    </button>
  </div>
</template>
