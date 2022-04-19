<template>
  <div>
    <NextJobs v-if="secretStore.secretAgent" />
    <CurrentJobs v-else />
  </div>
</template>

<script setup lang="ts">
const secretStore = useSecretStore();
</script>
