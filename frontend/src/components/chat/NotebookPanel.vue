<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { apiClient } from "@/api/client";
import { useGameStore } from "@/stores/game";

const gameStore = useGameStore();

// Dynamic word counts per country (only for countries with players)
const wordCounts = ref<Record<string, number>>({});

// State for each country's guesses (dynamic based on actual banned word count)
const notes = ref<Record<string, string[]>>({});

const loading = ref(true);
const submitting = ref(false);
const submitResult = ref<{
  success: boolean;
  discovered_count: number;
  total_required: number;
  victory_achieved: boolean;
} | null>(null);

// Get countries that have active players
const activeCountries = computed(() => {
  if (!gameStore.roomState?.participants) return [];
  return [
    ...new Set(gameStore.roomState.participants.map((p) => p.country)),
  ].sort();
});

const canSubmit = computed(() => {
  // Check if all fields for active countries are filled
  return activeCountries.value.every((country) => {
    const countryNotes = notes.value[country];
    return countryNotes && countryNotes.every((note) => note.trim().length > 0);
  });
});

const totalWords = computed(() =>
  Object.values(wordCounts.value).reduce((sum, count) => sum + count, 0),
);

// Fetch room info to get actual banned word counts
async function fetchRoomInfo() {
  try {
    loading.value = true;

    // Wait a bit for room to be created if needed
    await new Promise((resolve) => setTimeout(resolve, 500));

    const response: any = await apiClient.get(
      `/rooms/${gameStore.currentRoomId}/info`,
    );

    console.log("[Notebook] Room info received:", response);

    // Update word counts based on actual banned words
    const bannedWords = response.banned_words as Record<string, string[]>;
    const newNotes: Record<string, string[]> = {};
    const newCounts: Record<string, number> = {};

    // Only initialize notes for countries with active players
    for (const country of activeCountries.value) {
      if (bannedWords[country]) {
        const count = bannedWords[country].length;
        newCounts[country] = count;
        newNotes[country] = Array(count).fill("");
        console.log(`[Notebook] Country ${country}: ${count} words`);
      }
    }

    wordCounts.value = newCounts;
    notes.value = newNotes;
    console.log("[Notebook] Updated wordCounts:", newCounts);
  } catch (error: any) {
    console.error("Failed to fetch room info:", error);
    // If 404, room might not exist yet - retry once
    if (error.status === 404) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      try {
        const response: any = await apiClient.get(
          `/rooms/${gameStore.currentRoomId}/info`,
        );
        const bannedWords = response.banned_words as Record<string, string[]>;
        const newNotes: Record<string, string[]> = {};
        const newCounts: Record<string, number> = {};
        for (const country of activeCountries.value) {
          if (bannedWords[country]) {
            newCounts[country] = bannedWords[country].length;
            newNotes[country] = Array(bannedWords[country].length).fill("");
          }
        }
        wordCounts.value = newCounts;
        notes.value = newNotes;
      } catch (retryError) {
        console.error("Retry failed:", retryError);
      }
    }
  } finally {
    loading.value = false;
  }
}

// Watch for participant changes and update notes accordingly
watch(activeCountries, (newCountries, oldCountries) => {
  if (JSON.stringify(newCountries) === JSON.stringify(oldCountries)) return;

  console.log("[Notebook] Active countries changed:", newCountries);

  // Refetch room info when participants change
  if (gameStore.currentRoomId && newCountries.length > 0) {
    fetchRoomInfo();
  }
});

onMounted(() => {
  console.log("[Notebook] onMounted, currentRoomId:", gameStore.currentRoomId);
  if (gameStore.currentRoomId) {
    fetchRoomInfo();
  } else {
    loading.value = false;
  }
});

async function submitNotes() {
  if (!canSubmit.value) {
    alert("Please fill in all fields");
    return;
  }

  submitting.value = true;
  submitResult.value = null;

  try {
    const payload = {
      notes: Object.fromEntries(
        Object.entries(notes.value)
          .filter(([country]) => activeCountries.value.includes(country))
          .map(([country, words]) => [country, words.map((w) => w.trim())]),
      ),
    };

    const response: any = await apiClient.post(
      `/rooms/${gameStore.currentRoomId}/submit_notes`,
      payload,
      {
        "X-User-Token": gameStore.playerToken,
      },
    );

    submitResult.value = response;
  } catch (error) {
    console.error("Failed to submit notes:", error);
    alert("Failed to submit notes. Please try again.");
  } finally {
    submitting.value = false;
  }
}

function clearNotes() {
  const cleared: Record<string, string[]> = {};
  for (const [country, count] of Object.entries(wordCounts.value)) {
    cleared[country] = Array(count).fill("");
  }
  notes.value = cleared;
  submitResult.value = null;
}
</script>

<template>
  <div class="notebook-panel bg-[var(--tg-bg-secondary)] rounded-lg p-4 shadow-md border border-[var(--tg-bg-chat)]">
    <div class="mb-4">
      <h3 class="text-lg font-bold text-theme-accent">📝 Banned Words Notebook</h3>
    </div>

    <div v-if="loading" class="text-center py-8 text-theme-secondary">
      Loading...
    </div>

    <div v-else-if="activeCountries.length === 0" class="text-center py-8 text-theme-secondary">
      Waiting for players to join...
    </div>

    <div v-else class="text-sm text-theme-secondary mb-4">
      Guess {{ totalWords }} banned words for {{ activeCountries.length }} active {{ activeCountries.length === 1 ? 'country' : 'countries' }}:
    </div>

    <div v-if="!loading && activeCountries.length > 0" class="space-y-3">
      <div v-for="country in activeCountries" :key="country" class="country-section flex items-center gap-3">
        <div class="w-8 h-8 rounded-full bg-theme-accent flex items-center justify-center font-bold text-white flex-shrink-0">
          {{ country }}
        </div>
        
        <div class="flex-1 grid gap-2 grid-cols-1">
          <input
            v-for="i in wordCounts[country]"
            :key="i"
            v-model="(notes as any)[country][i - 1]"
            type="text"
            :placeholder="`Word ${i}`"
            class="bg-[var(--tg-bg)] border border-[var(--tg-bg-chat)] rounded px-4 py-3 text-theme-primary text-base placeholder-theme-secondary focus:outline-none focus:border-theme-accent transition"
          />
        </div>
      </div>
    </div>

    <div v-if="!loading && activeCountries.length > 0" class="flex gap-3 mt-6">
      <button
        @click="submitNotes"
        :disabled="!canSubmit || submitting || activeCountries.length === 0"
        class="flex-1 bg-theme-accent hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed text-white font-bold py-3 px-6 rounded-lg transition"
      >
        {{ submitting ? 'Submitting...' : 'Submit Guesses' }}
      </button>
    </div>

    <div v-if="!loading && activeCountries.length > 0" class="text-xs text-theme-secondary mt-3 text-center">
      When all {{ activeCountries.length }} {{ activeCountries.length === 1 ? 'player discovers' : 'players discover' }} all {{ totalWords }} banned words, the game is won!
    </div>
  </div>
</template>

<style scoped>
.notebook-panel {
  max-height: 80vh;
  overflow-y: auto;
}
</style>
