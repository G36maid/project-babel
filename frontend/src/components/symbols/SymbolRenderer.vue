<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useSymbols } from "@/composables/useSymbols";

const props = withDefaults(
  defineProps<{
    word: string;
    size?: number;
  }>(),
  {
    size: 24,
  },
);

const svgContent = ref<string | null>(null);
const loadError = ref(false);

const symbolConfig = computed(() => {
  return useSymbols().getSymbol(props.word);
});

onMounted(async () => {
  // Try to load SVG if symbol config exists
  if (symbolConfig.value && symbolConfig.value.svg) {
    try {
      // Fetch SVG from public folder (symbols.json paths already include "symbols/")
      const svgPath = `/${symbolConfig.value.svg}`;
      const response = await fetch(svgPath);
      if (!response.ok) {
        throw new Error(
          `Failed to fetch SVG: ${response.status} ${response.statusText}`,
        );
      }
      svgContent.value = await response.text();
    } catch (error) {
      console.error(`Failed to load SVG for "${props.word}":`, error);
      loadError.value = true;
    }
  }
});

const renderMode = computed(() => {
  // Fallback chain: SVG → emoji → text
  if (symbolConfig.value && svgContent.value && !loadError.value) {
    return "svg";
  }
  if (symbolConfig.value && symbolConfig.value.emoji) {
    return "emoji";
  }
  return "text";
});
</script>

<template>
  <span 
    class="symbol-renderer inline-block"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      lineHeight: `${size}px`
    }"
  >
    <!-- SVG rendering (primary) -->
    <span
      v-if="renderMode === 'svg'"
      v-html="svgContent"
      class="svg-container inline-block w-full h-full"
      :style="{
        color: 'currentColor'
      }"
    />
    
    <!-- Emoji fallback (secondary) -->
    <span
      v-else-if="renderMode === 'emoji'"
      class="emoji-container inline-block text-center"
      :style="{
        fontSize: `${size * 0.8}px`
      }"
    >
      {{ symbolConfig?.emoji }}
    </span>
    
    <!-- Text fallback (tertiary) -->
    <span
      v-else
      class="text-container inline-block text-center text-xs font-mono"
      :style="{
        fontSize: `${Math.max(size * 0.4, 10)}px`
      }"
    >
      {{ word }}
    </span>
  </span>
</template>

<style scoped>
.symbol-renderer {
  vertical-align: middle;
}

.svg-container :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}

.emoji-container,
.text-container {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
