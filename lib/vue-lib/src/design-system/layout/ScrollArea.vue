<template>
  <div class="flex flex-col absolute inset-0">
    <div v-if="$slots.top" class="shrink-0">
      <slot name="top" />
    </div>
    <div
      ref="scrollDivRef"
      :class="
        clsx(
          'overflow-auto flex-grow relative scroll-slot',
          hideScrollbar && 'scrollbar-hidden',
        )
      "
    >
      <slot />
    </div>
    <div v-if="$slots.bottom" class="shrink-0">
      <slot name="bottom" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { ref } from "vue";

defineProps({
  hideScrollbar: { type: Boolean },
});

const scrollDivRef = ref<HTMLElement | null>(null);
defineExpose({ scrollElement: scrollDivRef });
</script>
