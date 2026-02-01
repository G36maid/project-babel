<script setup lang="ts">
import { computed } from "vue";
import type { ConnectionState } from "@/types/websocket";

const props = defineProps<{
  state: ConnectionState;
}>();

const colorClass = computed(
  () =>
    ({
      idle: "bg-gray-500",
      connecting: "bg-yellow-500",
      connected: "bg-green-500",
      disconnected: "bg-red-500",
      error: "bg-red-600",
    })[props.state],
);

const label = computed(
  () =>
    ({
      idle: "Offline",
      connecting: "Connecting...",
      connected: "Online",
      disconnected: "Disconnected",
      error: "Error",
    })[props.state],
);
</script>

<template>
  <div class="flex items-center gap-2">
    <span class="text-xs text-[var(--tg-text-secondary)]">{{ label }}</span>
    <div :class="['w-2 h-2 rounded-full', colorClass]"></div>
  </div>
</template>
