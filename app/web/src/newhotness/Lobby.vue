<template>
  <div
    class="absolute w-screen h-screen bg-neutral-900 z-100 flex flex-col items-center justify-center"
  >
    <!-- Floating panel (to hold future animated border  -->
    <div
      :class="
        clsx(
          'w-[720px] max-w-[70vw] h-[400px] rounded shadow-[0_0_8px_0_rgba(255,255,255,0.08)]',
          'transition-opacity',
          showPanel ? 'opacity-100' : 'opacity-0',
        )
      "
    >
      <!-- Inner section of panel -->
      <div
        :class="
          clsx(
            'w-full h-full rounded font-mono text-sm',
            'flex flex-col justify-end gap-xs',
            // Note(Victor): We need borders, since padding does not hide overflow
            'overflow-hidden border-[16px] border-b-[64px]',
            'bg-neutral-800 border-neutral-800',
          )
        "
      >
        <LobbyTerminalOutputLine
          v-for="(data, index) in visibleSentences"
          :key="index"
          :message="data.sentence"
          :isActive="index === visibleSentences.length - 1"
          :isLoader="data.isLoader"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, watch } from "vue";
import clsx from "clsx";
import { sleep } from "@si/ts-lib/src/async-sleep";
import LobbyTerminalOutputLine from "@/newhotness/LobbyTerminalOutputLine.vue";

const sentences = [
  { sentence: "Welcome to System Initiative!" },
  { sentence: "Unpacking your workspace... it’s been through a lot" },
  { sentence: "We'll need a few minutes to reconstruct the state of things" },
  {
    sentence: "_______————***————_______".repeat(3), // There
    isLoader: true,
  },
  { sentence: "Loading change sets and parsing intent" },
  {
    sentence:
      "Scanning your workspace to see what's real... and what wants to be",
  },
  { sentence: "Populating attributes from source of truth" },
  { sentence: "Preparing views with just the right perspective" },
  { sentence: "Warming up the data so everything connects on the first try" },
  {
    sentence: "Filling in component attributes, thoughtfully and declaratively",
  },
  { sentence: "Views loading... perfect clarity takes a second" },
  {
    sentence:
      "Describing infrastructure so well, it might describe itself back",
  },
  { sentence: "Locating intent in a sea of configuration" },
  { sentence: "Applying structure without limiting flexibility" },
  { sentence: "Bringing your workspace to you — clean, clear, and traceable" },
  { sentence: "Finalizing..." },
];

const showPanel = ref(false);
const visibleSentenceCount = ref<number>(0);
const visibleSentences = computed(() =>
  sentences.slice(0, visibleSentenceCount.value),
);

// Delay before opening, so we don't blink the screen
const BLINK_DELAY_MS = 500;
// Delay so we complete the fade in before starting to show log lines
const TRANSITION_DELAY_MS = 200;

const kickOffTerminalLogs = async () => {
  await sleep(BLINK_DELAY_MS);
  showPanel.value = true;
  await sleep(TRANSITION_DELAY_MS);
  visibleSentenceCount.value = 1;
};

onMounted(kickOffTerminalLogs);

// Every time we show a sentence, enqueue showing the next sentence after a delay
const LOG_MIN_DELAY_MS = 4000;
const logDelayVariable = () =>
  LOG_MIN_DELAY_MS + Math.floor(Math.random() * 1000);
watch([visibleSentenceCount], async () => {
  if (!showPanel.value) return;

  if (visibleSentenceCount.value >= sentences.length) return;

  await sleep(logDelayVariable());

  visibleSentenceCount.value += 1;
});
</script>
