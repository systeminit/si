<template>
  <div class="container">
    <p class="text-md">Incoming Changes Merging...</p>
    <p class="text-xs">Applied by: {{ props.username }}</p>
  </div>
</template>

<script lang="ts" setup>
const props = defineProps<{
  username: string;
}>();
</script>
