<template>
  <span
    v-tooltip.bottom="tooltipText"
    :class="classes"
    :style="{ color: props.color }"
    :aria-label="props.tooltipText"
  >
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";

const props = defineProps<{
  color?: string;
  tooltipText: string;
}>();
const { tooltipText } = toRefs(props);

const classes = computed(() => {
  const results: Record<string, boolean> = {
    block: true,
    "w-6": true,
    "h-6": true,
    "text-gray-300": true,
    "hover:text-gray-100": true,
  };
  return results;
});
</script>
