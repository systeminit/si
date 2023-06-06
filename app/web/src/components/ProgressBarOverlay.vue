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
      <div class="grow whitespace-nowrap">
        {{ title }}
      </div>
      <slot name="detail">
        <div
          :class="
            clsx(
              'text-neutral-400 dark:text-neutral-500 text-sm font-normal pl-10 truncate',
            )
          "
        >
          {{ detail }}
        </div>
      </slot>

      <VButton
        v-if="featureFlagsStore.SINGLE_MODEL_SCREEN"
        icon="refresh"
        variant="ghost"
        loading-icon="refresh-active"
        loading-text="Refreshing..."
        :loading="refreshing"
        class="ml-2"
        @click="onClickRefreshButton"
      >
        Resources
      </VButton>
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
import { ref, computed, onBeforeUnmount } from "vue";
import { Icon, VButton } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useComponentsStore } from "@/store/components.store";
import ProgressBar from "./ProgressBar.vue";

const componentsStore = useComponentsStore();
const featureFlagsStore = useFeatureFlagsStore();

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

const refreshing = ref(false);
let timeout: Timeout;
const onClickRefreshButton = () => {
  refreshing.value = true;
  componentsStore.REFRESH_ALL_RESOURCE_INFO();
  timeout = setTimeout(() => {
    refreshing.value = false;
  }, 3000);
};

onBeforeUnmount(() => {
  clearTimeout(timeout);
});
</script>
