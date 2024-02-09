<template>
  <div
    class="absolute z-10 flex flex-col min-h-full items-center justify-center left-0 right-0 pointer-events-none select-none"
  >
    <div class="flex flex-row justify-evenly">
      <DiagramEmptyStateItem :number="1" />
      <DiagramEmptyStateItem :number="2" />
      <DiagramEmptyStateItem :number="3" />
      <DiagramEmptyStateItem :number="4" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import DiagramEmptyStateItem from "./DiagramEmptyStateItem.vue";
</script>
