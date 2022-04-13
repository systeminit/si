<template>
  <div>
    <NextHome v-if="secretStore.secretAgent" />
    <CurrentHome v-else />
  </div>
</template>

<script setup lang="ts">
const secretStore = useSecretStore();
</script>
