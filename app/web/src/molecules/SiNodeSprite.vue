<template>
  <div
    class="flex flex-row items-center gap-2.5 px-4 py-4 dark:border-[#525252] border-b-2 text-xs hover:bg-[#2F80ED] dark:text-white hover:text-white"
  >
    <NodeSkeleton :color="color" class="flex-shrink-0" />
    <span class=""> {{ name }} </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  color: string;
  name?: string;
}>();

import NodeSkeleton from "@/atoms/NodeSkeleton.vue";
</script>
