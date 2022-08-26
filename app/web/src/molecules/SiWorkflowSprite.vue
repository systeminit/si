<template>
  <div
    class="flex flex-row items-center gap-2.5 px-4 py-4 text-xs relative"
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
  color: string;
  name?: string;
  class?: string;
}>();

const classes = computed(() => props.class);
</script>
