<template>
  <div class="container">
    <p class="text-md">Changes Merged! Loading Workspace (head)</p>
    <p class="text-xs">{{ props.name }}</p>
  </div>
</template>

<script lang="ts" setup>
const props = defineProps<{
  name: string;
}>();
</script>
