<template>
  <div class="flex flex-row items-center gap-2.5 px-4 py-4 text-xs" :class="classes">
    <NodeSkeleton :color="props.color" class="flex-shrink-0" />
    <span class=""> {{ props.name }} </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import NodeSkeleton from "./NodeSkeleton.vue";

const props = defineProps<{
  color: string;
  name?: string;
  class?: string;
}>();

const classes = computed(() => props.class);
</script>
