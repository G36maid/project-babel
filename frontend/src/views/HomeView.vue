<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function joinGame() {
  router.push('/game')
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center">
    <h1 class="mb-8 text-6xl font-bold tracking-tight">
      🗼 Project Babel
    </h1>
    <p class="mb-8 text-center text-xl text-gray-400">
      In a world of surveillance, four strangers use a forgotten language to speak the truth.
    </p>
    <button
      @click="joinGame"
      class="rounded-lg bg-blue-600 px-8 py-4 text-2xl font-bold text-white transition-colors hover:bg-blue-700"
    >
      Join Game
    </button>
  </div>
</template>
