<template>
  <NodeSkeleton :color="color" />
  <span> {{ name }} </span>
</template>

<script setup lang="ts">
defineProps<{
  color: string;
  name?: string;
}>();

import NodeSkeleton from "@/atoms/NodeSkeleton.vue";
</script>
