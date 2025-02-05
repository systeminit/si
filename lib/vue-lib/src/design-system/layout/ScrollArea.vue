<template>
  <div class="flex flex-col absolute inset-0">
    <div v-if="$slots.top" class="shrink-0">
      <slot name="top" />
    </div>
    <div
      ref="scrollDivRef"
      class="overflow-auto flex-grow relative scroll-slot"
    >
      <slot />
    </div>
    <div v-if="$slots.bottom" class="shrink-0">
      <slot name="bottom" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";

const scrollDivRef = ref<HTMLElement | null>(null);
defineExpose({ scrollElement: scrollDivRef });
</script>
