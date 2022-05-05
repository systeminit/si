<template>
  <span
    v-tooltip.bottom="tooltipText"
    :class="classes"
    :style="{ color: props.color }"
    :aria-label="props.tooltipText"
    :disabled="props.disabled"
  >
    <slot></slot>
  </span>
</template>

<script setup lang="ts">
import { computed, toRefs } from "vue";

const props = defineProps<{
  disabled?: boolean;
  color?: string;
  tooltipText: string;
}>();
const { disabled, tooltipText } = toRefs(props);

const classes = computed(() => {
  const results: Record<string, boolean> = {
    block: true,
    "w-5": true,
    "h-5": true,
    "text-gray-300": true,
    "hover:text-gray-100": true,
  };
  if (disabled?.value) {
    results["opacity-50"] = true;
    results["cursor-not-allowed"] = true;
  }
  return results;
});
</script>
