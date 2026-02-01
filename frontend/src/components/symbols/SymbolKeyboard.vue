<script setup lang="ts">
import { computed, ref } from "vue";
import { useSymbols } from "@/composables/useSymbols";
import { useGameStore } from "@/stores/game";
import SymbolRenderer from "./SymbolRenderer.vue";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  select: [word: string];
  close: [];
}>();

const { getAllSymbols } = useSymbols();
const gameStore = useGameStore();

const hoveredWord = ref<string | null>(null);
const keyboardHeight = ref(192); // Default 192px (12rem / max-h-48)
const isDragging = ref(false);
const startY = ref(0);
const startHeight = ref(0);

// Get all available symbols from composable, filtered by allowed words
const allSymbols = computed(() => {
  const symbolsRecord = getAllSymbols();
  const allSymbolWords = Object.keys(symbolsRecord);

  // If no allowed words are set yet, show all symbols (fallback behavior)
  if (gameStore.allowedWords.length === 0) {
    return allSymbolWords;
  }

  // Filter symbols to only show allowed words
  return allSymbolWords.filter((word) => gameStore.allowedWords.includes(word));
});

function onSymbolClick(word: string) {
  emit("select", word);
}

function onClose() {
  emit("close");
}

function startDrag(event: MouseEvent) {
  isDragging.value = true;
  startY.value = event.clientY;
  startHeight.value = keyboardHeight.value;
  document.addEventListener("mousemove", onDrag);
  document.addEventListener("mouseup", stopDrag);
  event.preventDefault();
}

function onDrag(event: MouseEvent) {
  if (!isDragging.value) return;

  const deltaY = startY.value - event.clientY;
  const newHeight = Math.max(100, Math.min(600, startHeight.value + deltaY));
  keyboardHeight.value = newHeight;
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
}
</script>

<template>
  <div 
    v-if="visible"
    class="bg-[var(--tg-bg-secondary)] border-t border-[var(--tg-bg-chat)]"
  >
    <!-- Resize Handle -->
    <div 
      @mousedown="startDrag"
      class="h-2 cursor-ns-resize flex items-center justify-center hover:bg-[var(--tg-bg-chat)] transition-colors"
      :class="{ 'bg-[var(--tg-bg-chat)]': isDragging }"
    >
      <div class="w-12 h-1 bg-[var(--tg-text-secondary)] rounded-full opacity-50"></div>
    </div>

    <div class="flex justify-between items-center py-1 px-2">
      <span class="text-[var(--tg-text-secondary)] text-xs">Symbols</span>
      <button 
        @click="onClose"
        class="text-[var(--tg-text-secondary)] hover:text-[var(--tg-text)] px-1.5 py-0.5 text-sm"
      >
        ✕
      </button>
    </div>
    
    <!-- Compact Grid with Adjustable Height -->
    <div 
      class="overflow-x-auto overflow-y-auto px-2 pb-2 pt-10 relative"
      :style="{ height: `${keyboardHeight}px` }"
    >
      <div class="flex flex-wrap gap-1">
        <button
          v-for="word in allSymbols"
          :key="word"
          @click="onSymbolClick(word)"
          @mouseenter="hoveredWord = word"
          @mouseleave="hoveredWord = null"
          class="w-20 h-20 flex-shrink-0 flex items-center justify-center bg-[var(--tg-bg)] hover:bg-[var(--tg-bg-chat)] rounded transition-colors relative group"
        >
          <SymbolRenderer :word="word" :size="20" />
          <div 
            v-if="hoveredWord === word"
            class="absolute -top-9 left-1/2 -translate-x-1/2 bg-gray-900 text-white text-xs px-2 py-1 rounded whitespace-nowrap z-10 pointer-events-none shadow-lg"
          >
            {{ word }}
          </div>
        </button>
      </div>
    </div>
  </div>
</template>
