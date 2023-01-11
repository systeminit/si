<template>
  <div
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="classes"
  >
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ props.name }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  name?: string;
  class?: string;
}>();

const classes = computed(() => props.class);
</script>
