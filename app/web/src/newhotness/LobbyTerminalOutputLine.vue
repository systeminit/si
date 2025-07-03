<template>
  <div class="flex flex-row items-center">
    <Icon
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
        {{ croppedSentence }}
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
  /// Shows SI logo on the left side of line and in active colors, if false color will be greyer
  isActive: { type: Boolean },
  /// When loader, will be shown in secondary greyer color, and without the initial delay
  isLoader: { type: Boolean },
});

const visibleCharCount = ref(-1);
const croppedSentence = computed(() =>
  props.message.slice(0, Math.max(visibleCharCount.value, 0)),
);

// Kick-off line animation
const startWritingSentence = async () => {
  visibleCharCount.value = 0;
};
onMounted(startWritingSentence);

// Write character one at a time, enqueuing the next change with a watch that sleeps
const INITIAL_DELAY_MS = 800;
const CHAR_DELAY_MS = 35;
const BASE_ELLIPSIS_DELAY_MS = 300;
const ellipsisDelay = () =>
  BASE_ELLIPSIS_DELAY_MS + Math.floor(Math.random() * 200);
watch([visibleCharCount], async () => {
  const message = props.message ?? "";
  if (visibleCharCount.value >= message.length) return;

  const latestChar = message[visibleCharCount.value - 1] ?? "";

  let delay = CHAR_DELAY_MS;

  // Add some extra delay on the initial character, except if the line is a loader
  if (!props.isLoader && visibleCharCount.value === 0) {
    delay = INITIAL_DELAY_MS;
  } else if (latestChar === ".") {
    delay = ellipsisDelay();
  }

  await sleep(delay);

  visibleCharCount.value += 1;
});
</script>
