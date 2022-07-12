<template>
  <ChevronDownIcon aria-hidden="true" :class="arrowNudged(props.nudge)" />
</template>

<script setup lang="ts">
import { ChevronDownIcon } from "@heroicons/vue/solid";

const props = defineProps<{
  nudge?: boolean;
}>();

const arrowNudged = (nudge: boolean | undefined): Record<string, boolean> => {
  if (nudge) {
    return {
      "mt-1": true,
    };
  }
  return {};
};
</script>
