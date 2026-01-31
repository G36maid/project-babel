<script setup lang="ts">
</script>

<template>
  <div class="flex flex-col h-full bg-[var(--tg-bg-chat)]">
    <!-- Header Slot -->
    <div class="flex-shrink-0">
      <slot name="header" />
    </div>
    
    <!-- Messages Slot - Scrollable -->
    <div class="flex-1 overflow-y-auto min-h-0">
      <slot name="messages" />
    </div>
    
    <!-- Input Slot -->
    <div class="flex-shrink-0">
      <slot name="input" />
    </div>
  </div>
</template>
