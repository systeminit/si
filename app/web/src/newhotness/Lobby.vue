<template>
  <div
    data-testid="lobby"
    class="absolute w-screen h-screen bg-neutral-900 z-[1000] flex flex-col items-center justify-center"
  >
    <!-- Floating panel (holds box shadow)  -->
    <div
      :class="
        clsx(
          'w-[720px] max-w-[70vw] h-[400px] rounded bg-neutral-800',
          'shadow-[0_0_8px_0_rgba(255,255,255,0.08)]',
          'transition-opacity',
          showPanel ? 'opacity-100' : 'opacity-0',
        )
      "
    >
      <!-- Holds spinning border (we can't use the internal one because of margins -->
      <div class="spinning-border h-full w-full flex flex-row rounded">
        <!-- Inner section of panel -->
        <div
          :class="
            clsx(
              'rounded font-mono text-sm ',
              'flex flex-col justify-end gap-xs ',
              'overflow-hidden m-sm mb-xl',
              'bg-neutral-800 border-neutral-800',
            )
          "
        >
          <LobbyTerminalOutputLine
            v-for="(data, index) in visibleSentences"
            :key="index"
            :message="unref(data.sentence)"
            :isActive="index === visibleSentences.length - 1"
            :isLoader="data.isLoader"
            :isLastElement="index === visibleSentences.length - 1"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, unref, watch } from "vue";
import clsx from "clsx";
import { sleep } from "@si/ts-lib/src/async-sleep";
import LobbyTerminalOutputLine from "@/newhotness/LobbyTerminalOutputLine.vue";

const props = defineProps({
  /// The number of loading steps that exist and whether they are complete
  loadingSteps: { type: Array<boolean>, required: true },
});

const loadingSummaryText = computed(() => {
  const totalLoadingSteps = props.loadingSteps.length;
  const completedLoadingSteps = props.loadingSteps.filter(
    (f) => f === true,
  ).length;

  return `${completedLoadingSteps}/${totalLoadingSteps}`;
});

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
  { sentence: computed(() => `Finalizing... (${loadingSummaryText.value})`) },
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

<style scoped>
/* Note*(victor): These styles exist to power the spinning border on the lobby terminal */

/*
  We need to declare --angle as a property with an initial value so that keyframes can correctly interpolate it
  If we don't, the keyframes below would only blip between the declared states
 */
@property --angle {
  syntax: "<angle>";
  inherits: false;
  initial-value: 0deg;
}
@keyframes borderRotate {
  100% {
    --angle: 360deg;
  }
}

.spinning-border {
  border: 1px solid;
  /*
    use a spinning conic-gradient as the border image. It looks like this: https://www.geeksforgeeks.org/css/css-conic-gradient-function/
    But "masked" through the border
  */
  border-image: conic-gradient(
      from var(--angle),
      #333,
      #333 0.95turn,
      #aaa8 1turn
    )
    1;
  /*
    Enable the animation. Although the rotation is linear, since it's showing through a rectangular shape,
    both the moving speed and the trail length vary depending on the position. We could fudge the border speed by
    compensating on the keyframes vs the "radius" of each border position but this wouldn't fix the trail so
    we chose to use this as is.
  */
  animation: borderRotate 9000ms linear infinite forwards;
  /* border-radius does not interact with border images, so this all black mask that takes the size of the div makes it round again */
  mask-image: radial-gradient(#000 0, #000 0);
}
</style>
