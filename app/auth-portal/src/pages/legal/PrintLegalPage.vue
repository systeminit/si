<template>
  <h1>print some stuff...</h1>
</template>

<script setup lang="ts">
import { onMounted } from "vue";

onMounted(() => {
  if (import.meta.env.SSR) return;

  window.print();
  window.onfocus = () => {
    window.close();
  };
});
</script>
