<template>
  <div>The approval process for {{ action }} this change set has been cancelled.</div>
</template>

<script lang="ts" setup>
defineProps({
  action: { type: String, required: true },
});
</script>
