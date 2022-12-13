<template>
  <div
    :class="
      clsx(
        'absolute z-20 left-0 right-0 mx-4 mt-3 p-4',
        'bg-white dark:bg-neutral-800 dark:text-white border border-neutral-300 dark:border-neutral-600',
        'shadow-md rounded-md font-bold',
      )
    "
  >
    <div class="flex justify-between items-center">
      <span>
        {{ title }}
        <slot name="detail">
          <span
            :class="
              clsx(
                'text-neutral-400 dark:text-neutral-500 text-sm font-normal pl-10',
              )
            "
            >{{ detail }}
          </span>
        </slot>
      </span>
    </div>

    <Transition
      enter-active-class="duration-300 ease-out"
      enter-from-class="transform opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="delay-1000 duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="transform opacity-0"
    >
      <div
        v-show="computedProgressPercent < 1"
        v-if="computedProgressPercent !== undefined"
        class="mt-2 flex gap-5"
      >
        <ProgressBar :progress-percent="computedProgressPercent" />
        <span class="whitespace-nowrap flex-shrink-0">
          <slot name="barLabel">
            <template v-if="totalCount === Infinity">
              <Icon name="loader" class="-mt-1" />
            </template>
            <template v-else>
              {{ doneCount }} / {{ totalCount }} {{ barLabel }}
            </template>
          </slot>
        </span>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed } from "vue";
import ProgressBar from "@/atoms/ProgressBar.vue";
import Icon from "@/ui-lib/icons/Icon.vue";

const props = defineProps({
  title: { type: String },
  detail: { type: String },
  // set the X/Y label and unless overridden, the progress bar fill
  doneCount: { type: Number },
  totalCount: { type: Number },
  // can set to override bar fill
  progressPercent: { type: Number },
  barLabel: { type: String },
});

const computedProgressPercent = computed(() => {
  if (props.progressPercent !== undefined) return props.progressPercent;
  if (props.totalCount !== undefined) {
    if (props.totalCount === 0) return undefined;
    return (props.doneCount || 0) / props.totalCount;
  }
  return undefined;
});
</script>
