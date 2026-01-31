<script setup lang="ts">
import { RouterView } from 'vue-router'
import { useTheme } from '@/composables/useTheme'

// Initialize theme
useTheme()
</script>

<template>
  <div id="app" class="min-h-screen bg-[var(--tg-bg)] text-[var(--tg-text)] w-full">
    <RouterView />
  </div>
</template>
