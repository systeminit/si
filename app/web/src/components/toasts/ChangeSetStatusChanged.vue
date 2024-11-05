<template>
  <div>{{ user }} has {{ command }} {{ changeSetName }}</div>
</template>

<script lang="ts" setup>
defineProps({
  user: { type: String, required: true },
  changeSetName: { type: String, required: true },
  command: { type: String, required: true },
});
</script>
