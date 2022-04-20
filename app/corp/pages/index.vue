<template>
  <div class="flex w-full">
    <NextHome v-if="secretStore.secretAgent" />
    <CurrentHome v-else />
  </div>
</template>

<script setup lang="ts">
const secretStore = useSecretStore();
</script>
