<script setup lang="ts">
const props = defineProps<{
  sidebarOpen?: boolean;
}>();

const emit = defineEmits<{
  "close-sidebar": [];
}>();
</script>

<template>
  <div class="flex h-screen w-full overflow-hidden bg-[var(--tg-bg)] relative">
    <!-- Desktop Sidebar (md+) -->
    <div class="hidden md:block w-80 flex-shrink-0 h-full border-r border-[var(--tg-bg-chat)] z-10">
      <slot name="sidebar" />
    </div>

    <!-- Mobile Sidebar Drawer -->
    <div class="md:hidden fixed inset-0 z-50 pointer-events-none">
      <!-- Backdrop -->
      <div 
        v-if="sidebarOpen"
        @click="emit('close-sidebar')"
        class="absolute inset-0 bg-black/50 pointer-events-auto transition-opacity duration-300"
      ></div>

      <!-- Drawer -->
      <div 
        class="absolute left-0 top-0 bottom-0 w-[280px] bg-[var(--tg-bg)] transition-transform duration-300 ease-out pointer-events-auto shadow-2xl"
        :class="sidebarOpen ? 'translate-x-0' : '-translate-x-full'"
      >
        <slot name="sidebar" />
      </div>
    </div>
    
    <!-- Chat Area -->
    <div class="flex-1 h-full min-w-0 bg-[var(--tg-bg)] relative z-0">
      <slot name="chat" />
    </div>
  </div>
</template>
