<script setup lang="ts">
</script>

<template>
  <div class="flex h-screen w-full overflow-hidden bg-[var(--tg-bg)]">
    <!-- Sidebar - Hidden on mobile, visible on md+ -->
    <div class="hidden md:block w-80 flex-shrink-0 h-full">
      <slot name="sidebar" />
    </div>
    
    <!-- Chat Area - Full width on mobile, flex-1 on desktop -->
    <div class="flex-1 h-full min-w-0">
      <slot name="chat" />
    </div>
  </div>
</template>
