<template>
  <div class="flex flex-row items-center">
    <Icon
      v-if="props.prependSiLogo"
      name="logo-si"
      size="sm"
      :class="clsx('mr-xs self-start', !isActive && 'opacity-0')"
    />
    <div class="leading-[1.5rem] tracking-tight">
      <span
        :class="
          clsx(!isLoader && isActive ? 'text-neutral-100' : 'text-neutral-400')
        "
      >
        {{ cropSentence ? croppedSentence : message }}
      </span>
      <div
        v-if="isActive"
        :class="
          clsx(
            // Since this is an inline cursor like element, there's some finicky positioning required to make it look good
            'w-xs h-sm inline-block relative top-0.5 left-0.5',
            'bg-white animate-pulse [animation-duration:_1s]',
          )
        "
      ></div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { computed, onMounted, ref, watch } from "vue";
import clsx from "clsx";
import { sleep } from "@si/ts-lib/src/async-sleep";

const props = defineProps({
  message: { type: String, required: true },
  // On active lines, show SI logo before it. If not active, reserve some space for it for alignment.
  prependSiLogo: { type: Boolean },
  // Shows SI logo to the left of the line (if enabled) and draws text in active colors. If false, color will be greyer
  isActive: { type: Boolean },
  // When loader, will be shown in secondary greyer color, and without the initial delay
  isLoader: { type: Boolean },
  // When the user goes out of the tab then comes back, we have an issue where multiple lines will be animating at the same time.
  // this is because this component is mounted only at that time. This flag prevents that by forcing animate only on the last line.
  isLastElement: { type: Boolean },
  // Scroll the characters very fast, without the initial delay.
  fast: { type: Boolean },
});

const visibleCharCount = ref(-1);
const croppedSentence = computed(() =>
  props.message.slice(0, Math.max(visibleCharCount.value, 0)),
);
// Fuse so we don't animate when message contents change after we're done rendering.
const finishedRendering = ref(false);
const cropSentence = computed(
  () => props.isLastElement && !finishedRendering.value,
);

// Kick-off line animation
const startWritingSentence = async () => {
  visibleCharCount.value = 0;
};
onMounted(startWritingSentence);

// Write character one at a time, enqueuing the next change with a watch that sleeps
const INITIAL_DELAY_MS = 800;
const CHAR_DELAY_MS = props.fast ? 15 : 35;
const BASE_ELLIPSIS_DELAY_MS = 300;
const ellipsisDelay = () =>
  BASE_ELLIPSIS_DELAY_MS + Math.floor(Math.random() * 100);
watch([visibleCharCount], async () => {
  const message = props.message ?? "";
  if (visibleCharCount.value >= message.length) {
    finishedRendering.value = true;
    return;
  }

  const latestChar = message[visibleCharCount.value - 1] ?? "";

  let delay = CHAR_DELAY_MS;

  // Add some extra delay on the initial character, except if the line is a loader
  if (!props.fast && !props.isLoader && visibleCharCount.value === 0) {
    delay = INITIAL_DELAY_MS;
  } else if (latestChar === ".") {
    delay = ellipsisDelay();
  }

  await sleep(delay);

  visibleCharCount.value += 1;
});
</script>
