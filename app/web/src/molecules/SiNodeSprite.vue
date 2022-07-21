<template>
  <div
    class="flex flex-row items-center gap-2.5 px-4 py-4 text-xs"
    :class="classes"
  >
    <NodeSkeleton :color="color" class="flex-shrink-0" />
    <span class=""> {{ name }} </span>
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

import NodeSkeleton from "@/atoms/NodeSkeleton.vue";
</script>
